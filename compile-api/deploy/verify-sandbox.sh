#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# verify-sandbox.sh - prove, on the deployed host, that player code cannot
# read host files at build time and that the two-unit deployment is wired
# the way the security review requires. Run as root, before exposing the
# service and after every update:
#
#     sudo /opt/pnc-compile/verify-sandbox.sh
#
# It requires POSITIVE evidence for every claim and prints PASS only if
# every single check held; any failed check exits non-zero.
#
#   1. Secrets: the API env file is 0600 root:root and unreadable by both
#      service users; the builder unit and its env file hold no tokens.
#   2. Units: both are active; the builder journal (current run) shows
#      "isolation checks passed" and "build self-test passed"; the API
#      journal shows it hands builds to the builder.
#   3. Sandbox probes, run as the builder user with exactly the builder's
#      settings, through `pledge_compile_api sandbox-verify` (deny-list OFF
#      for that one CLI build; it has no listener, refuses to run with a
#      token in its environment and refuses PNC_BUILD_SANDBOX=none, so it
#      cannot be "left on"):
#        - positive control: include_str! of a file that IS inside the
#          sandbox (/build/Cargo.toml) compiles, runs and prints it - so
#          the probe mechanism (absolute include, deny-list really off)
#          works;
#        - host control: every probed path EXISTS on the host;
#        - each negative probe must fail with rustc's exact
#          "couldn't read `<path>`: No such file or directory" (or
#          "file not found for module" / "not defined at compile time"),
#          not merely fail.
#   4. Config refusals (no network): `serve` with a token and
#      PNC_BUILD_SANDBOX=none, `builder` with a token, and `sandbox-verify`
#      with a token must all refuse to start.
#   5. HTTP through the real API with exact status codes: /health 200,
#      no token 401, an end-to-end build 200 with the program's output,
#      a deny-listed probe 400 source_policy, /metrics on the API
#      listener 404, /metrics on the loopback metrics listener 200 (or
#      401 without its token when PNC_METRICS_TOKENS is set).
#
# The probe sources below are the only file-read payloads in the
# repository; they live here because the check is meaningless without them.

set -euo pipefail

BIN="${PNC_BIN:-/usr/local/bin/pledge_compile_api}"
API_ENV="${PNC_ENV_FILE:-/etc/pnc-compile/env}"
BUILDER_ENV="${PNC_BUILDER_ENV:-/etc/pnc-builder/builder.env}"
API_USER="${PNC_API_USER:-pnc-api}"
BUILDER_USER="${PNC_BUILDER_USER:-pnc-builder}"
API_UNIT="${PNC_API_UNIT:-pnc-compile-api}"
BUILDER_UNIT="${PNC_BUILDER_UNIT:-pnc-builder}"
SOCKET="${PNC_BUILDER_SOCKET_PATH:-/run/pnc-builder/builder.sock}"

PASSED=0
FAILED=0
ok()   { PASSED=$((PASSED + 1)); echo "ok   - $*"; }
fail() { FAILED=$((FAILED + 1)); echo "FAIL - $*" >&2; }
die()  { echo "FAIL - $*" >&2; echo "RESULT: FAIL (aborted)" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || die "run as root (sudo)"
for tool in curl runuser systemctl journalctl grep sed mktemp stat install head od tr timeout; do
    command -v "$tool" >/dev/null || die "$tool is required"
done
[ -x "$BIN" ] || die "$BIN is not executable"
[ -f "$API_ENV" ] || die "API env file $API_ENV not found"
[ -f "$BUILDER_ENV" ] || die "builder env file $BUILDER_ENV not found"

WORK="$(mktemp -d /tmp/pnc-verify.XXXXXX)"
chmod 0711 "$WORK"
trap 'rm -rf "$WORK"' EXIT
rand_hex() { head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n'; }

# ---- 1. secrets ----------------------------------------------------------------
perms="$(stat -c '%a %U %G' "$API_ENV")"
if [ "$perms" = "600 root root" ]; then ok "$API_ENV is 600 root root"; else fail "$API_ENV is '$perms', expected '600 root root'"; fi
for user in "$API_USER" "$BUILDER_USER"; do
    if runuser -u "$user" -- cat "$API_ENV" >/dev/null 2>&1; then
        fail "$user can read $API_ENV"
    else
        ok "$user cannot read $API_ENV"
    fi
done
if grep -Eq '^PNC_(AUTH|METRICS)_TOKENS=.+' "$BUILDER_ENV"; then
    fail "$BUILDER_ENV contains a token"
else
    ok "$BUILDER_ENV holds no token"
fi
if systemctl cat "$BUILDER_UNIT" | grep -v '^[[:space:]]*#' | grep -q '/etc/pnc-compile'; then
    fail "$BUILDER_UNIT references /etc/pnc-compile (the secret env file)"
else
    ok "$BUILDER_UNIT does not reference the secret env file"
fi
if grep -q '^PNC_BUILD_SANDBOX=bwrap$' "$BUILDER_ENV"; then
    ok "builder uses PNC_BUILD_SANDBOX=bwrap"
else
    die "$BUILDER_ENV must set PNC_BUILD_SANDBOX=bwrap"
fi

# ---- 2. units --------------------------------------------------------------------
for unit in "$BUILDER_UNIT" "$API_UNIT"; do
    if systemctl is-active --quiet "$unit"; then ok "$unit is active"; else fail "$unit is not active"; fi
done
journal_of() {
    local inv
    inv="$(systemctl show -p InvocationID --value "$1")"
    [ -n "$inv" ] || { echo ""; return; }
    journalctl --no-pager -o cat _SYSTEMD_INVOCATION_ID="$inv" 2>/dev/null || true
}
BUILDER_LOG="$(journal_of "$BUILDER_UNIT")"
for needle in "toolchain: matches the pinned version" "isolation checks passed" "build self-test passed" "builder: listening"; do
    if grep -qF "$needle" <<<"$BUILDER_LOG"; then ok "builder journal: '$needle'"; else fail "builder journal lacks '$needle'"; fi
done
API_LOG="$(journal_of "$API_UNIT")"
if grep -qF "executor: builder service" <<<"$API_LOG"; then
    ok "API journal: builds go to the builder service"
else
    fail "API journal lacks 'executor: builder service'"
fi
if [ -S "$SOCKET" ]; then ok "builder socket $SOCKET exists"; else fail "builder socket $SOCKET missing"; fi

# ---- 3. sandbox probes -------------------------------------------------------------
mapfile -t BENV < <(grep -E '^[A-Z_][A-Z0-9_]*=' "$BUILDER_ENV")
PROC_MODE="$(sed -n 's/^PNC_BWRAP_PROC=//p' "$BUILDER_ENV" | tail -n 1)"
PROBE_ROOT="$WORK/root"
install -d -m 0700 -o "$BUILDER_USER" "$PROBE_ROOT"
CANARY="PNC_PROBE_CANARY"
CANARY_VALUE="canary-$(rand_hex)"
API_PID="$(systemctl show -p MainPID --value "$API_UNIT")"

N=0
# run_probe <source>: run one probe build as the builder user; JSON on stdout.
run_probe() {
    N=$((N + 1))
    local src="$WORK/probe-$N.rs"
    printf '%s\n' "$1" > "$src"
    runuser -u "$BUILDER_USER" -- env -i PATH=/usr/bin:/bin "${BENV[@]}" \
        PNC_BUILD_ROOT="$PROBE_ROOT" RUST_LOG=warn "$CANARY=$CANARY_VALUE" \
        "$BIN" sandbox-verify < "$src" 2> "$WORK/probe-$N.err"
}

# Positive control: a file that IS visible inside the sandbox.
if out="$(run_probe 'fn main() { print!("{}", include_str!("/build/Cargo.toml")); }')"; then
    if grep -qF '"compiled":true' <<<"$out" && grep -qF '"ran_ok":true' <<<"$out" \
        && grep -qF 'player_submission' <<<"$out"; then
        ok "positive control: include_str!(\"/build/Cargo.toml\") read the in-sandbox file"
    else
        fail "positive control did not read /build/Cargo.toml (probe mechanism broken): $out"
    fi
else
    fail "positive control: sandbox-verify failed to run: $(head -c 800 "$WORK/probe-$N.err")"
fi

# probe_missing <name> <path> <source>: path exists on the host, and rustc
# inside the sandbox reports exactly that it does not exist.
probe_missing() {
    local name="$1" path="$2" src="$3" out
    if [ ! -e "$path" ]; then
        fail "host control: $path does not exist on the host, probe '$name' would prove nothing"
        return
    fi
    if ! out="$(run_probe "$src")"; then
        fail "probe '$name': sandbox-verify failed to run: $(head -c 800 "$WORK/probe-$N.err")"
        return
    fi
    if grep -qF "$CANARY_VALUE" <<<"$out"; then
        fail "probe '$name' LEAKED the environment canary"
    elif grep -qF '"compiled":false' <<<"$out" \
        && grep -qF "couldn't read \`$path\`: No such file or directory" <<<"$out"; then
        ok "probe '$name': rustc in the sandbox: couldn't read \`$path\`: No such file or directory"
    else
        fail "probe '$name': expected rustc \"couldn't read \`$path\`: No such file or directory\", got: $(head -c 800 <<<"$out")"
    fi
}

probe_missing "API env file (include_str)" "$API_ENV" \
    "fn main() { print!(\"{}\", include_str!(\"$API_ENV\")); }"
probe_missing "/etc/passwd (include_bytes)" "/etc/passwd" \
    'fn main() { let b: &[u8] = include_bytes!("/etc/passwd"); println!("{}", b.len()); }'
probe_missing "builder env file" "$BUILDER_ENV" \
    "fn main() { print!(\"{}\", include_str!(\"$BUILDER_ENV\")); }"
probe_missing "builder socket" "$SOCKET" \
    "fn main() { let b: &[u8] = include_bytes!(\"$SOCKET\"); println!(\"{}\", b.len()); }"
if [ -n "$API_PID" ] && [ "$API_PID" != "0" ]; then
    probe_missing "API process environ" "/proc/$API_PID/environ" \
        "fn main() { let b: &[u8] = include_bytes!(\"/proc/$API_PID/environ\"); println!(\"{}\", b.len()); }"
else
    fail "no MainPID for $API_UNIT; cannot probe its /proc entry"
fi
if [ "${PROC_MODE:-none}" = "none" ]; then
    probe_missing "/proc/self/environ (no procfs in the sandbox)" "/proc/self/environ" \
        'fn main() { let b: &[u8] = include_bytes!("/proc/self/environ"); println!("{}", b.len()); }'
fi

# #[path] module redirection: rustc E0583 with the module name.
if out="$(run_probe "#[path = \"$API_ENV\"] mod leak; fn main() {}")"; then
    if grep -qF '"compiled":false' <<<"$out" && grep -qF 'file not found for module `leak`' <<<"$out"; then
        ok "probe '#[path] to the env file': file not found for module \`leak\`"
    else
        fail "probe '#[path]': expected 'file not found for module \`leak\`', got: $(head -c 800 <<<"$out")"
    fi
else
    fail "probe '#[path]': sandbox-verify failed to run"
fi

# Build-time environment: the canary IS in sandbox-verify's own
# environment, so env! would compile if the environment leaked in.
if out="$(run_probe "fn main() { println!(\"{}\", env!(\"$CANARY\")); }")"; then
    if grep -qF "$CANARY_VALUE" <<<"$out"; then
        fail "probe 'env!': the build saw the caller's environment"
    elif grep -qF '"compiled":false' <<<"$out" \
        && grep -qF "environment variable \`$CANARY\` not defined at compile time" <<<"$out"; then
        ok "probe 'env!': environment variable \`$CANARY\` not defined at compile time"
    else
        fail "probe 'env!': expected 'not defined at compile time', got: $(head -c 800 <<<"$out")"
    fi
else
    fail "probe 'env!': sandbox-verify failed to run"
fi

# ---- 4. config refusals (no network) -------------------------------------------------
TOK="$(rand_hex)"
refuses() {
    local name="$1" needle="$2"; shift 2
    local out
    if out="$(timeout 20 runuser -u "$BUILDER_USER" -- env -i PATH=/usr/bin:/bin RUST_LOG=error "$@" 2>&1)"; then
        fail "$name: started (exit 0) but must refuse"
    elif grep -qF "$needle" <<<"$out"; then
        ok "$name: refused ($needle)"
    else
        fail "$name: exited non-zero without the expected refusal '$needle': $(head -c 600 <<<"$out")"
    fi
}
refuses "serve with a token and PNC_BUILD_SANDBOX=none" "refusing" \
    PNC_AUTH_TOKENS="$TOK" PNC_BUILD_SANDBOX=none PNC_BIND=127.0.0.1:1 PNC_METRICS_BIND=127.0.0.1:2 \
    "$BIN" serve
refuses "builder with a token" "must hold no secrets" \
    PNC_AUTH_TOKENS="$TOK" "${BENV[@]}" "$BIN" builder
refuses "sandbox-verify with a token" "must hold no secrets" \
    PNC_AUTH_TOKENS="$TOK" "${BENV[@]}" "$BIN" sandbox-verify

# ---- 5. HTTP through the real API ----------------------------------------------------
get_var() { sed -n "s/^$1=//p" "$API_ENV" | tail -n 1 | tr -d '[:space:]'; }
TOKEN="$(get_var PNC_AUTH_TOKENS | cut -d, -f1)"
METRICS_TOKEN="$(get_var PNC_METRICS_TOKENS | cut -d, -f1)"
BIND="$(get_var PNC_BIND)"; BIND="${BIND:-127.0.0.1:7878}"
METRICS_BIND="$(get_var PNC_METRICS_BIND)"; METRICS_BIND="${METRICS_BIND:-127.0.0.1:9878}"
[ -n "$TOKEN" ] || fail "PNC_AUTH_TOKENS is empty in $API_ENV (production requires it)"

# expect_status <name> <code> <curl args...>: exact status or FAIL.
expect_status() {
    local name="$1" want="$2" code; shift 2
    code="$(curl -sS -o "$WORK/body" -w '%{http_code}' --max-time 120 "$@" 2>"$WORK/curl.err")" \
        || code="curl-error($(head -c 200 "$WORK/curl.err"))"
    if [ "$code" = "$want" ]; then
        ok "$name: HTTP $code"
        return 0
    fi
    fail "$name: expected HTTP $want, got $code: $(head -c 400 "$WORK/body" 2>/dev/null)"
    return 1
}
JSON='content-type: application/json'
expect_status "GET /health" 200 "http://$BIND/health" || true
expect_status "POST /compile-real without a token" 401 -X POST -H "$JSON" \
    -d '{"source":"fn main(){}","encounter_id":"pnc_verify"}' "http://$BIND/compile-real" || true
if expect_status "POST /compile-real end to end (API -> builder -> job)" 200 \
    -H "Authorization: Bearer $TOKEN" -H "$JSON" \
    -d '{"source":"fn main(){ println!(\"pnc-e2e-ok\"); }","encounter_id":"pnc_verify"}' \
    "http://$BIND/compile-real"; then
    if grep -qF '"ok":true' "$WORK/body" && grep -qF 'pnc-e2e-ok' "$WORK/body"; then
        ok "end-to-end build ran and returned the program's output"
    else
        fail "end-to-end build: 200 but no ok/output: $(head -c 400 "$WORK/body")"
    fi
fi
if expect_status "deny-list is on in production" 400 \
    -H "Authorization: Bearer $TOKEN" -H "$JSON" \
    -d '{"source":"fn main(){ let _ = include_str!(\"/etc/passwd\"); }","encounter_id":"pnc_verify"}' \
    "http://$BIND/compile-real"; then
    if grep -qF '"error":"source_policy"' "$WORK/body"; then ok "deny-list refusal is source_policy"; else fail "400 but not source_policy"; fi
fi
expect_status "/metrics is not served on the API listener" 404 "http://$BIND/metrics" || true
expect_status "/metrics through the API listener with a proxy header" 404 \
    -H 'X-Forwarded-For: 203.0.113.9' "http://$BIND/metrics" || true
if [ -n "$METRICS_TOKEN" ]; then
    expect_status "metrics listener without its token" 401 "http://$METRICS_BIND/metrics" || true
    expect_status "metrics listener with its token" 200 \
        -H "Authorization: Bearer $METRICS_TOKEN" "http://$METRICS_BIND/metrics" || true
else
    expect_status "metrics listener (direct loopback)" 200 "http://$METRICS_BIND/metrics" || true
fi

# ---- verdict -------------------------------------------------------------------------
if [ "$FAILED" -eq 0 ] && [ "$PASSED" -gt 0 ]; then
    echo "PASS ($PASSED checks held, 0 failed)"
    exit 0
fi
echo "RESULT: FAIL ($FAILED failed, $PASSED passed)" >&2
exit 1
