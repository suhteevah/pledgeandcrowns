#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# verify-sandbox.sh - prove, on the deployed VPS, that player source cannot
# read host files at compile time through the compile API.
#
# Run as root on the server, against the LOCAL API (127.0.0.1), before the
# service is exposed and after every update:
#
#     sudo /opt/pnc-compile/verify-sandbox.sh
#
# What it does:
#   1. Checks the env file is 0600 root:root and unreadable by the service
#      user.
#   2. Backs up the env file, appends a random canary variable, and turns
#      the second-layer source deny-list OFF (PNC_SOURCE_DENYLIST=false) so
#      the probes actually reach rustc and test the sandbox itself.
#   3. Restarts the service (the canary is now in the server's environment
#      and therefore in /proc/<pid>/environ) and waits for /health.
#   4. Submits probe programs that try to read, at compile time, the env
#      file, the server's /proc/<pid>/environ, /proc/1/environ and /etc.
#   5. FAILS if the canary or the bearer token appears in ANY response, or
#      if a probe that must fail to compile succeeds. A positive control
#      checks that a normal program still builds and runs.
#   6. Always restores the original env file and restarts the service
#      (trap on EXIT), so the deny-list is back on afterwards.
#
# These probe sources are the only file-read payloads in the repository;
# they exist here because the check is meaningless without them.

set -euo pipefail

ENV_FILE="${PNC_ENV_FILE:-/etc/pnc-compile/env}"
UNIT="${PNC_UNIT:-compile-api}"
SERVICE_USER="${PNC_SERVICE_USER:-pnc-compile}"

die() { echo "FAIL: $*" >&2; exit 1; }
info() { echo "--- $*"; }

[ "$(id -u)" -eq 0 ] || die "run as root (sudo)"
command -v curl >/dev/null || die "curl is required"
[ -f "$ENV_FILE" ] || die "env file $ENV_FILE not found"

# ---- 1. env file permissions ------------------------------------------------
perms="$(stat -c '%a %U %G' "$ENV_FILE")"
[ "$perms" = "600 root root" ] || die "$ENV_FILE is '$perms', expected '600 root root'"
if runuser -u "$SERVICE_USER" -- cat "$ENV_FILE" >/dev/null 2>&1; then
    die "$SERVICE_USER can read $ENV_FILE"
fi
info "env file is root-only"

get_var() { sed -n "s/^$1=//p" "$ENV_FILE" | tail -n 1; }
TOKEN="$(get_var PNC_AUTH_TOKENS | cut -d, -f1 | tr -d '[:space:]')"
BIND="$(get_var PNC_BIND)"; BIND="${BIND:-127.0.0.1:7878}"
SANDBOX="$(get_var PNC_BUILD_SANDBOX)"
[ "$SANDBOX" = "bwrap" ] || die "PNC_BUILD_SANDBOX is '${SANDBOX:-unset}', expected 'bwrap'"
case "$BIND" in
    127.*|"[::1]"*) ;;
    *) die "PNC_BIND=$BIND is not loopback; run this against a loopback bind" ;;
esac
API="http://$BIND"

# ---- 2. canary + deny-list off (restored on exit) ---------------------------
BACKUP="$(mktemp /root/pnc-env-backup.XXXXXX)"
cp -p "$ENV_FILE" "$BACKUP"
restore() {
    status=$?
    cp -p "$BACKUP" "$ENV_FILE" && rm -f "$BACKUP"
    systemctl restart "$UNIT" || true
    if [ $status -eq 0 ]; then echo "PASS (env restored, $UNIT restarted)";
    else echo "env restored, $UNIT restarted; result: FAIL" >&2; fi
}
trap restore EXIT

CANARY="pnc-canary-$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')"
{
    echo ""
    echo "PNC_SANDBOX_CANARY=$CANARY"
    echo "PNC_SOURCE_DENYLIST=false"
} >> "$ENV_FILE"
grep -q "^PNC_SANDBOX_CANARY=$CANARY\$" "$ENV_FILE" || die "could not plant canary"

# ---- 3. restart and wait -----------------------------------------------------
info "restarting $UNIT with canary"
systemctl restart "$UNIT"
for _ in $(seq 1 60); do
    curl -fsS "$API/health" >/dev/null 2>&1 && break
    sleep 1
done
curl -fsS "$API/health" >/dev/null || die "$UNIT did not come back (journalctl -u $UNIT)"
PID="$(systemctl show -p MainPID --value "$UNIT")"
[ -n "$PID" ] && [ "$PID" != "0" ] || die "no MainPID for $UNIT"
grep -qa "$CANARY" "/proc/$PID/environ" || die "canary is not in the server environment; test invalid"
info "canary planted in env file and server environ (pid $PID)"

# ---- 4/5. probes --------------------------------------------------------------
json_escape() { printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'; }

submit() {
    local src="$1"
    local auth=()
    [ -n "$TOKEN" ] && auth=(-H "Authorization: Bearer $TOKEN")
    curl -sS --max-time 90 "${auth[@]}" -H 'content-type: application/json' \
        -d "{\"encounter_id\":\"sandbox_probe\",\"source\":\"$(json_escape "$src")\"}" \
        "$API/compile-real"
}

assert_clean() {
    local name="$1" resp="$2"
    if printf '%s' "$resp" | grep -qF "$CANARY"; then
        die "probe '$name' LEAKED THE CANARY: $resp"
    fi
    if [ -n "$TOKEN" ] && printf '%s' "$resp" | grep -qF "$TOKEN"; then
        die "probe '$name' LEAKED THE BEARER TOKEN"
    fi
}

assert_not_ok() {
    local name="$1" resp="$2"
    printf '%s' "$resp" | grep -q '"ok":false' || die "probe '$name' unexpectedly succeeded: $resp"
}

# Positive control: the sandboxed build works at all.
resp="$(submit 'fn main() { println!("sandbox-ok"); }')"
printf '%s' "$resp" | grep -q 'sandbox-ok' || die "positive control failed: $resp"
info "positive control: ok"

probe() {
    local name="$1" src="$2"
    local resp
    resp="$(submit "$src")"
    assert_clean "$name" "$resp"
    assert_not_ok "$name" "$resp"
    info "probe '$name': blocked"
}

probe "env file via include_str" \
    "fn main() { print!(\"{}\", include_str!(\"$ENV_FILE\")); }"
probe "env file via include_bytes" \
    "fn main() { let b: &[u8] = include_bytes!(\"$ENV_FILE\"); println!(\"{}\", String::from_utf8_lossy(b)); }"
probe "env file via #[path] diagnostics" \
    "#[path = \"$ENV_FILE\"] mod leak; fn main() {}"
probe "server environ via include_bytes" \
    "fn main() { let b: &[u8] = include_bytes!(\"/proc/$PID/environ\"); println!(\"{}\", String::from_utf8_lossy(b)); }"
probe "/etc visible at all" \
    "fn main() { print!(\"{}\", include_str!(\"/etc/hostname\")); }"

# /proc/1 inside the sandbox is bubblewrap's own init. It may be readable;
# what matters is that its environment holds nothing of the server's.
resp="$(submit 'fn main() { let b: &[u8] = include_bytes!("/proc/1/environ"); println!("{}", String::from_utf8_lossy(b)); }')"
assert_clean "sandbox pid 1 environ" "$resp"
info "probe 'sandbox pid 1 environ': no canary"

# Compile-time environment: env! must not see server variables.
resp="$(submit 'fn main() { println!("{}", env!("PNC_SANDBOX_CANARY")); }')"
assert_clean "env! canary" "$resp"
assert_not_ok "env! canary" "$resp"
info "probe 'env! canary': blocked"

exit 0
