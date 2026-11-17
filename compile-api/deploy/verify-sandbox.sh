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
#      journal shows it hands builds to the builder. The builder unit's key
#      security properties have their SHIPPED values (table SHIPPED below):
#      each exact line is in the installed unit file AND `systemctl show`
#      of the running builder reports the shipped value, so a loosened unit
#      (edited file or drop-in) fails here, not only a mismatch between the
#      probe service and the builder.
#   3. Sandbox probes through `pledge_compile_api sandbox-verify` (deny-list
#      OFF for that one CLI build; it has no listener, refuses to run with a
#      token in its environment and refuses PNC_BUILD_SANDBOX=none, so it
#      cannot be "left on"), in up to two passes (PNC_VERIFY_PROBES):
#        - `runuser`: as the builder user with the builder's env file. Tests
#          bwrap and the rlimits alone, outside any systemd sandboxing.
#        - `unit`: in a transient systemd service carrying the INSTALLED
#          builder unit's own [Service] settings (`systemctl cat`, drop-ins
#          included; only ExecStart/Type/Restart*/RuntimeDirectory* are
#          replaced). So the unit's seccomp filter, namespaces restrictions,
#          ProtectProc/ProtectSystem/PrivateTmp/NoExecPaths, capability
#          bounding set and cgroup limits are what the probes run under.
#          The running builder itself cannot run them: it applies the
#          deny-list to every request by design. Before the probes, the
#          transient service's effective security properties are compared
#          with the running builder's (`systemctl show`); any difference
#          fails the check.
#      Default `both`. In each pass:
#        - positive control: include_str! of a file that IS inside the
#          sandbox (/build/Cargo.toml) compiles, runs and prints it - so
#          the probe mechanism (absolute include, deny-list really off)
#          works;
#        - host control: every probed path EXISTS on the host;
#        - each negative probe must fail with rustc's exact
#          "couldn't read `<path>`: No such file or directory" (or, for
#          #[path] to a file the builder user CAN read on the host,
#          "couldn't find file `<path>`"; for env!, "not defined at
#          compile time"), not merely fail.
#        - `/proc/sys/kernel/overflowuid` is a path ProtectProc= does NOT
#          hide (bwrap itself reads it) but the sandbox (no procfs) does:
#          the [unit] pass first shows, with no bwrap, that the unit can
#          read it, so "No such file" in the sandbox can only come from
#          bwrap. (The API's /proc/<pid>/environ is hidden by ProtectProc
#          even without bwrap, so it is probed in the [runuser] pass only,
#          where it is not.)
#      RUN PHASE (the wasmtime run happens in a second, smaller bwrap
#      sandbox). A guest cannot be the prober, wasmtime confines it, so
#      `pledge_compile_api sandbox-verify-run` starts that exact sandbox with
#      the server-owned native probe `run-probe` in place of `run-wasm`, and
#      the probe reports (existence and connect results only, never file
#      contents):
#        - positive controls: it received its request on stdin; its own
#          binary is visible at /pnc/pledge_compile_api; a loopback connect
#          gets "refused" (it can make connections at all);
#        - every probed host path (both env files, /etc, /etc/passwd, /run,
#          the builder socket, /proc/self, /proc/sys/kernel/overflowuid,
#          the installed binary's host path, /usr/local, /usr/bin/bwrap, the
#          toolchain, /tmp, /var, /home, /root) is "not_found" (each EXISTS
#          on the host); the builder socket cannot be connected to; an
#          outside address is "network_unreachable";
#        - a new PID namespace (pid <= 3), the builder's uid, the run
#          rlimits (cpu 30, nofile 64, fsize 0, core 0), and an environment
#          with nothing but LD_LIBRARY_PATH, PWD (set by bwrap) and RUST_LOG
#          - not the canary;
#        - a listing of every file reachable from / (names only): exactly
#          the binary and its shared libraries (ld-linux, libc, libm,
#          libgcc_s).
#   4. Config refusals (no network): `serve` with a token and
#      PNC_BUILD_SANDBOX=none, `builder` with a token, and `sandbox-verify`
#      / `sandbox-verify-run` with a token must all refuse to start.
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
# Probe passes: runuser, unit or both (see the header).
VERIFY_PROBES="${PNC_VERIFY_PROBES:-both}"
case "$VERIFY_PROBES" in runuser|unit|both) ;; *) echo "PNC_VERIFY_PROBES must be runuser, unit or both" >&2; exit 2 ;; esac

PASSED=0
FAILED=0
ok()   { PASSED=$((PASSED + 1)); echo "ok   - $*"; }
fail() { FAILED=$((FAILED + 1)); echo "FAIL - $*" >&2; }
die()  { echo "FAIL - $*" >&2; echo "RESULT: FAIL (aborted)" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || die "run as root (sudo)"
for tool in curl runuser systemctl systemd-run journalctl busctl grep sed awk mktemp stat install head od tr timeout id; do
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

# Shipped values of the builder unit's key security properties:
#   "<systemctl show property>|<exact line in the unit file>|<show value>"
# A show value of `-` skips the show comparison (the property is checked by
# its unit-file line only, e.g. the expanded syscall list). Calibrated with
# systemd 257 (openSUSE Leap Micro 6.2); tests/deploy_files.rs checks that
# every line here is in deploy/pnc-builder.service.
SHIPPED=(
    "User|User=pnc-builder|pnc-builder"
    "Group|Group=pnc-ipc|pnc-ipc"
    "PrivateTmp|PrivateTmp=yes|yes"
    "NoExecPaths|NoExecPaths=/tmp|/tmp"
    "UMask|UMask=0077|0077"
    "TemporaryFileSystem|TemporaryFileSystem=/usr/local:ro|-"
    "TemporaryFileSystem|TemporaryFileSystem=/opt:ro|-"
    "TemporaryFileSystem|TemporaryFileSystem=/run:ro|/usr/local:ro /opt:ro /run:ro"
    "BindReadOnlyPaths|BindReadOnlyPaths=/usr/local/bin/pledge_compile_api|-"
    "BindReadOnlyPaths|BindReadOnlyPaths=/opt/pnc|/usr/local/bin/pledge_compile_api:/usr/local/bin/pledge_compile_api:rbind /opt/pnc:/opt/pnc:rbind"
    "BindPaths|BindPaths=/run/pnc-builder|/run/pnc-builder:/run/pnc-builder:rbind"
    "InaccessiblePaths|InaccessiblePaths=-/srv -/mnt -/media -/boot -/var/lib -/var/log -/var/cache -/var/spool -/var/mail -/var/opt -/var/www -/var/backups -/var/local|-/srv -/mnt -/media -/boot -/var/lib -/var/log -/var/cache -/var/spool -/var/mail -/var/opt -/var/www -/var/backups -/var/local"
    "ProtectSystem|ProtectSystem=strict|strict"
    "ProtectHome|ProtectHome=yes|yes"
    "PrivateDevices|PrivateDevices=yes|yes"
    "PrivateNetwork|PrivateNetwork=yes|yes"
    "ProtectKernelModules|ProtectKernelModules=yes|yes"
    "ProtectControlGroups|ProtectControlGroups=yes|yes"
    "ProtectClock|ProtectClock=yes|yes"
    "ProtectHostname|ProtectHostname=yes|yes"
    "ProtectKernelTunables|ProtectKernelTunables=yes|yes"
    "ProtectKernelLogs|ProtectKernelLogs=yes|yes"
    "ProtectProc|ProtectProc=invisible|invisible"
    "NoNewPrivileges|NoNewPrivileges=yes|yes"
    "CapabilityBoundingSet|CapabilityBoundingSet=|"
    "AmbientCapabilities|AmbientCapabilities=|"
    "RestrictSUIDSGID|RestrictSUIDSGID=yes|yes"
    "RestrictNamespaces|RestrictNamespaces=user mnt pid net ipc uts cgroup|-"
    "RestrictRealtime|RestrictRealtime=yes|yes"
    "LockPersonality|LockPersonality=yes|yes"
    "RemoveIPC|RemoveIPC=yes|yes"
    "SystemCallArchitectures|SystemCallArchitectures=native|native"
    "SystemCallFilter|SystemCallFilter=@system-service @mount|-"
    "SystemCallFilter|SystemCallFilter=~@clock @cpu-emulation @debug @module @obsolete @raw-io @reboot @swap|-"
    "SystemCallErrorNumber|SystemCallErrorNumber=EPERM|1"
    "RestrictAddressFamilies|RestrictAddressFamilies=AF_UNIX AF_NETLINK|AF_NETLINK AF_UNIX"
    "IPAddressDeny|IPAddressDeny=any|0.0.0.0/0 ::/0"
    "MemoryMax|MemoryMax=1536M|1610612736"
    "MemorySwapMax|MemorySwapMax=0|0"
    "OOMPolicy|OOMPolicy=continue|continue"
    "TasksMax|TasksMax=256|256"
    "LimitNOFILE|LimitNOFILE=4096|4096"
    "LimitCORE|LimitCORE=0|0"
    "CPUQuotaPerSecUSec|CPUQuota=200%|2s"
)
# Properties that must NOT be set on the builder (each would break bwrap or
# was deliberately left off; see the unit comments): their unit-file key.
NOT_SET=(ProcSubset PrivateUsers MemoryDenyWriteExecute)
FRAGMENT="$(systemctl show -p FragmentPath --value "$BUILDER_UNIT")"
if [ -z "$FRAGMENT" ] || [ ! -f "$FRAGMENT" ]; then
    fail "cannot find the installed unit file of $BUILDER_UNIT"
else
    UNIT_LINES="$(grep -v '^[[:space:]]*#' "$FRAGMENT" | sed 's/[[:space:]]*$//')"
    shipped_bad=0
    for entry in "${SHIPPED[@]}"; do
        prop="${entry%%|*}"; rest="${entry#*|}"; line="${rest%|*}"; want="${rest##*|}"
        if ! grep -qxF -- "$line" <<<"$UNIT_LINES"; then
            shipped_bad=1; fail "shipped value: '$line' is not in $FRAGMENT (edited or loosened?)"
        fi
        if [ "$want" != "-" ]; then
            got="$(systemctl show -p "$prop" --value "$BUILDER_UNIT")"
            # systemd 255 prints the IPAddressDeny list as `::/0 0.0.0.0/0`,
            # 257 as `0.0.0.0/0 ::/0`. The list is a set: compare it sorted.
            if [ "$prop" = "IPAddressDeny" ]; then
                got="$(tr ' ' '\n' <<<"$got" | sort | tr '\n' ' ' | sed 's/ $//')"
                want="$(tr ' ' '\n' <<<"$want" | sort | tr '\n' ' ' | sed 's/ $//')"
            fi
            if [ "$got" != "$want" ]; then
                shipped_bad=1; fail "shipped value: running $BUILDER_UNIT has $prop='$got', shipped '$want' (drop-in or edit?)"
            fi
        fi
    done
    # `systemctl show` prints RestrictNamespaces= for "everything but time"
    # as `no` (systemd 257), the same word as an unrestricted unit, so read
    # the raw allow mask over D-Bus instead: CLONE_NEWNS|NEWCGROUP|NEWUTS|
    # NEWIPC|NEWUSER|NEWPID|NEWNET = 0x7E020000 = 2114060288 (no NEWTIME).
    unit_path="$(busctl call org.freedesktop.systemd1 /org/freedesktop/systemd1 \
        org.freedesktop.systemd1.Manager GetUnit s "$BUILDER_UNIT.service" 2>/dev/null | awk '{print $2}' | tr -d '"')"
    ns="$(busctl get-property org.freedesktop.systemd1 "$unit_path" org.freedesktop.systemd1.Service \
        RestrictNamespaces 2>/dev/null | awk '{print $2}')"
    if [ "$ns" != "2114060288" ]; then
        shipped_bad=1; fail "shipped value: running $BUILDER_UNIT has RestrictNamespaces mask '$ns', shipped 2114060288 (user mnt pid net ipc uts cgroup)"
    fi
    for key in "${NOT_SET[@]}"; do
        if systemctl cat "$BUILDER_UNIT" | grep -v '^[[:space:]]*#' | grep -q "^$key="; then
            shipped_bad=1; fail "shipped value: $key= is set on $BUILDER_UNIT but must not be (see the unit comments)"
        fi
    done
    if [ "$shipped_bad" -eq 0 ]; then
        ok "builder unit has its shipped values (${#SHIPPED[@]} lines in $FRAGMENT and in systemctl show; ${#NOT_SET[@]} properties unset)"
    fi
fi

# ---- 3. sandbox probes -------------------------------------------------------------
mapfile -t BENV < <(grep -E '^[A-Z_][A-Z0-9_]*=' "$BUILDER_ENV")
PROC_MODE="$(sed -n 's/^PNC_BWRAP_PROC=//p' "$BUILDER_ENV" | tail -n 1)"
PROBE_ROOT="$WORK/root"
install -d -m 0700 -o "$BUILDER_USER" "$PROBE_ROOT"
CANARY="PNC_PROBE_CANARY"
CANARY_VALUE="canary-$(rand_hex)"
API_PID="$(systemctl show -p MainPID --value "$API_UNIT")"

PASS_LABEL=""
PROBE_MODE="runuser"
UNIT_PROPS=()

# The installed builder unit's [Service] settings, minus what a one-shot
# probe service must replace. Values are passed to systemd-run unchanged.
load_unit_props() {
    local line
    UNIT_PROPS=()
    while IFS= read -r line; do
        UNIT_PROPS+=(-p "$line")
    done < <(systemctl cat "$BUILDER_UNIT" | awk '
        /^[[:space:]]*#/ { next }
        /^\[/ { sec = $0; next }
        sec == "[Service]" && /^[A-Za-z]+=/ { print }' \
        | grep -Ev '^(Type|ExecStart|ExecStartPre|ExecStartPost|ExecReload|ExecStop|ExecStopPost|Restart|RestartSec|RestartPreventExitStatus|RuntimeDirectory|RuntimeDirectoryMode|RuntimeDirectoryPreserve|WatchdogSec|NotifyAccess|PIDFile|StandardInput|StandardOutput|StandardError)=')
    [ "${#UNIT_PROPS[@]}" -gt 20 ] || die "could not read the [Service] settings of $BUILDER_UNIT"
}

# Security-relevant properties that must be identical in the running
# builder and in the transient probe service.
COMPARE_PROPS="User Group SystemCallFilter SystemCallErrorNumber SystemCallArchitectures RestrictNamespaces ProtectProc ProcSubset PrivateTmp PrivateNetwork PrivateDevices ProtectSystem ProtectHome ProtectKernelTunables ProtectKernelLogs ProtectKernelModules ProtectControlGroups ProtectClock ProtectHostname NoNewPrivileges CapabilityBoundingSet AmbientCapabilities RestrictAddressFamilies IPAddressDeny NoExecPaths RestrictSUIDSGID RestrictRealtime LockPersonality MemoryDenyWriteExecute MemoryMax MemorySwapMax TasksMax CPUQuotaPerSecUSec LimitNOFILE LimitCORE UMask EnvironmentFiles OOMPolicy TemporaryFileSystem BindReadOnlyPaths BindPaths InaccessiblePaths PrivateUsers"

compare_unit_props() {
    local unit="pnc-verify-props-$$" prop a b same=1
    if ! systemd-run --quiet --unit="$unit" --remain-after-exit "${UNIT_PROPS[@]}" /usr/bin/true \
        2>"$WORK/props.err"; then
        fail "unit pass: systemd-run refused the builder unit's settings: $(head -c 600 "$WORK/props.err")"
        return 1
    fi
    for _ in 1 2 3 4 5 6 7 8 9 10; do
        [ "$(systemctl show -p ActiveState --value "$unit")" = "active" ] && break
        sleep 0.5
    done
    for prop in $COMPARE_PROPS; do
        a="$(systemctl show -p "$prop" --value "$BUILDER_UNIT")"
        b="$(systemctl show -p "$prop" --value "$unit")"
        if [ "$a" != "$b" ]; then
            same=0
            fail "unit pass: $prop differs (running builder '$a', probe service '$b')"
        fi
    done
    systemctl stop "$unit" >/dev/null 2>&1 || true
    systemctl reset-failed "$unit" >/dev/null 2>&1 || true
    if [ "$same" -eq 1 ]; then
        ok "unit pass: probe service has the running builder's settings ($(wc -w <<<"$COMPARE_PROPS") security properties identical)"
        return 0
    fi
    return 1
}

# run_probe <source>: run one probe build; JSON on stdout, its stderr in
# $LAST_ERR. It always runs inside $(...) (a subshell), so nothing it sets
# survives: each call names its files from a fresh mktemp instead of a
# counter, and copies its stderr to the fixed $LAST_ERR path.
LAST_ERR="$WORK/last-probe.err"
run_probe() {
    local src err id
    src="$(mktemp "$WORK/probe-XXXXXX")"
    id="${src##*/probe-}"
    err="$WORK/probe-$id.err"
    printf '%s\n' "$1" > "$src"
    if [ "$PROBE_MODE" = "unit" ]; then
        # The unit's EnvironmentFile wins over -E for the same key (systemd
        # rule), so only keys the env file does not set are added here.
        # stdin/stdout/stderr are pipes, not files: with --pipe, systemd
        # (init_t) receives these descriptors, and SELinux policy lets it
        # use the caller's pipes but not write a user_tmp_t file (seen as
        # permissive-mode AVCs on the first host).
        printf '%s\n' "$1" | timeout 300 systemd-run --quiet --wait --pipe --collect \
            --unit="pnc-verify-probe-$$-$id" "${UNIT_PROPS[@]}" -p RuntimeMaxSec=240 \
            -E PNC_BUILD_ROOT=/tmp/pnc-verify-probe -E "$CANARY=$CANARY_VALUE" \
            "$BIN" sandbox-verify 2> >(cat > "$err")
    else
        runuser -u "$BUILDER_USER" -- env -i PATH=/usr/bin:/bin "${BENV[@]}" \
            PNC_BUILD_ROOT="$PROBE_ROOT" RUST_LOG=warn "$CANARY=$CANARY_VALUE" \
            "$BIN" sandbox-verify < "$src" 2> "$err"
    fi
    local rc=$?
    wait 2>/dev/null
    cp -f "$err" "$LAST_ERR" 2>/dev/null
    return $rc
}

sandbox_probes() {
local out
# Positive control: a file that IS visible inside the sandbox.
if out="$(run_probe 'fn main() { print!("{}", include_str!("/build/Cargo.toml")); }')"; then
    if grep -qF '"compiled":true' <<<"$out" && grep -qF '"ran_ok":true' <<<"$out" \
        && grep -qF 'player_submission' <<<"$out"; then
        ok "$PASS_LABEL positive control: include_str!(\"/build/Cargo.toml\") read the in-sandbox file"
    else
        fail "$PASS_LABEL positive control did not read /build/Cargo.toml (probe mechanism broken): $out"
    fi
else
    fail "$PASS_LABEL positive control: sandbox-verify failed to run: $(head -c 800 "$LAST_ERR")"
fi

probe_missing "API env file (include_str)" "$API_ENV" \
    "fn main() { print!(\"{}\", include_str!(\"$API_ENV\")); }"
probe_missing "/etc/passwd (include_bytes)" "/etc/passwd" \
    'fn main() { let b: &[u8] = include_bytes!("/etc/passwd"); println!("{}", b.len()); }'
probe_missing "builder env file" "$BUILDER_ENV" \
    "fn main() { print!(\"{}\", include_str!(\"$BUILDER_ENV\")); }"
probe_missing "builder socket" "$SOCKET" \
    "fn main() { let b: &[u8] = include_bytes!(\"$SOCKET\"); println!(\"{}\", b.len()); }"
# Visible under ProtectProc= (bwrap reads it at startup; the [unit] pass
# proves the unit itself can read it), absent only because of bwrap.
probe_missing "/proc/sys/kernel/overflowuid (not hidden by ProtectProc)" \
    "/proc/sys/kernel/overflowuid" \
    'fn main() { let b: &[u8] = include_bytes!("/proc/sys/kernel/overflowuid"); println!("{}", b.len()); }'
if [ "$PROBE_MODE" = "runuser" ]; then
    # Only meaningful without ProtectProc=, which hides it with or without
    # bwrap: here the host answers "Permission denied", the sandbox "No such
    # file or directory".
    if [ -n "$API_PID" ] && [ "$API_PID" != "0" ]; then
        probe_missing "API process environ" "/proc/$API_PID/environ" \
            "fn main() { let b: &[u8] = include_bytes!(\"/proc/$API_PID/environ\"); println!(\"{}\", b.len()); }"
    else
        fail "no MainPID for $API_UNIT; cannot probe its /proc entry"
    fi
fi
if [ "${PROC_MODE:-none}" = "none" ]; then
    probe_missing "/proc/self/environ (no procfs in the sandbox)" "/proc/self/environ" \
        'fn main() { let b: &[u8] = include_bytes!("/proc/self/environ"); println!("{}", b.len()); }'
fi

# #[path] module redirection. rustc checks only that the module file
# EXISTS, and a stat that fails with EACCES (the 0700 /etc/pnc-compile)
# also reads as "not found", so the target must be a file the builder
# user CAN read on the host: then "not found" can only mean the sandbox.
# Wording: rustc 1.99 says "couldn't find file `<path>`"; older releases
# said "file not found for module `leak`" (E0583).
if ! runuser -u "$BUILDER_USER" -- test -r "$BUILDER_ENV"; then
    fail "host control: $BUILDER_USER cannot read $BUILDER_ENV, the #[path] probe would prove nothing"
elif out="$(run_probe "#[path = \"$BUILDER_ENV\"] mod leak; fn main() {}")"; then
    if grep -qF '"compiled":false' <<<"$out" \
        && { grep -qF "couldn't find file \`$BUILDER_ENV\`" <<<"$out" \
             || grep -qF 'file not found for module `leak`' <<<"$out"; }; then
        ok "$PASS_LABEL probe '#[path] to the (host-readable) builder env file': rustc in the sandbox cannot find it"
    else
        fail "$PASS_LABEL probe '#[path]': expected \"couldn't find file \`$BUILDER_ENV\`\", got: $(head -c 800 <<<"$out")"
    fi
else
    fail "$PASS_LABEL probe '#[path]': sandbox-verify failed to run"
fi

# Build-time environment: the canary IS in sandbox-verify's own
# environment, so env! would compile if the environment leaked in.
if out="$(run_probe "fn main() { println!(\"{}\", env!(\"$CANARY\")); }")"; then
    if grep -qF "$CANARY_VALUE" <<<"$out"; then
        fail "$PASS_LABEL probe 'env!': the build saw the caller's environment"
    elif grep -qF '"compiled":false' <<<"$out" \
        && grep -qF "environment variable \`$CANARY\` not defined at compile time" <<<"$out"; then
        ok "$PASS_LABEL probe 'env!': environment variable \`$CANARY\` not defined at compile time"
    else
        fail "$PASS_LABEL probe 'env!': expected 'not defined at compile time', got: $(head -c 800 <<<"$out")"
    fi
else
    fail "$PASS_LABEL probe 'env!': sandbox-verify failed to run"
fi
}

# probe_missing <name> <path> <source>: path exists on the host, and rustc
# inside the sandbox reports exactly that it does not exist.
probe_missing() {
    local name="$1" path="$2" src="$3" out
    if [ ! -e "$path" ]; then
        fail "host control: $path does not exist on the host, probe '$name' would prove nothing"
        return
    fi
    if ! out="$(run_probe "$src")"; then
        fail "$PASS_LABEL probe '$name': sandbox-verify failed to run: $(head -c 800 "$LAST_ERR")"
        return
    fi
    if grep -qF "$CANARY_VALUE" <<<"$out"; then
        fail "$PASS_LABEL probe '$name' LEAKED the environment canary"
    elif grep -qF '"compiled":false' <<<"$out" \
        && grep -qF "couldn't read \`$path\`: No such file or directory" <<<"$out"; then
        ok "$PASS_LABEL probe '$name': rustc in the sandbox: couldn't read \`$path\`: No such file or directory"
    else
        fail "$PASS_LABEL probe '$name': expected rustc \"couldn't read \`$path\`: No such file or directory\", got: $(head -c 800 <<<"$out")"
    fi
}

# ---- run phase -------------------------------------------------------------------
TOOLCHAIN_DIR="$(sed -n 's/^PNC_TOOLCHAIN_DIR=//p' "$BUILDER_ENV" | tail -n 1)"
RUN_PATHS=("$API_ENV" "$BUILDER_ENV" /etc /etc/passwd /run "$SOCKET" /proc/self
           /proc/sys/kernel/overflowuid "$BIN" /usr/local /usr/bin/bwrap "$TOOLCHAIN_DIR"
           /tmp /var /home /root)
BUILDER_UID="$(id -u "$BUILDER_USER")"

run_probe_request() {
    local p sep="" json='{"paths":['
    for p in "${RUN_PATHS[@]}"; do json="$json$sep\"$p\""; sep=","; done
    printf '%s],"unix_socket":"%s","tcp_target":"1.1.1.1:443","tcp_control":"127.0.0.1:9","walk_root":true}' "$json" "$SOCKET"
}

# Run the native probe in the run sandbox; its result lines on stdout.
run_phase_probe() {
    local req err="$WORK/run-probe-$PROBE_MODE.err"
    req="$(run_probe_request)"
    if [ "$PROBE_MODE" = "unit" ]; then
        printf '%s' "$req" | timeout 120 systemd-run --quiet --wait --pipe --collect \
            --unit="pnc-verify-run-$$" "${UNIT_PROPS[@]}" -p RuntimeMaxSec=90 \
            -E "$CANARY=$CANARY_VALUE" "$BIN" sandbox-verify-run 2> >(cat > "$err")
    else
        printf '%s' "$req" | runuser -u "$BUILDER_USER" -- env -i PATH=/usr/bin:/bin "${BENV[@]}" \
            RUST_LOG=warn "$CANARY=$CANARY_VALUE" "$BIN" sandbox-verify-run 2> "$err"
    fi
    local rc=$?
    wait 2>/dev/null
    cp -f "$err" "$LAST_ERR" 2>/dev/null
    return $rc
}

# expect_line <name> <exact JSON line fragment> <probe output>
expect_line() {
    if grep -qF -- "$2" <<<"$3"; then
        ok "$PASS_LABEL run phase: $1"
    else
        fail "$PASS_LABEL run phase: $1: expected '$2' in: $(head -c 1200 <<<"$3")"
    fi
}

run_phase_probes() {
    local out p v
    for p in "${RUN_PATHS[@]}"; do
        [ -e "$p" ] || fail "host control: $p does not exist on the host, its run-phase probe would prove nothing"
    done
    if ! out="$(run_phase_probe)"; then
        fail "$PASS_LABEL run phase: sandbox-verify-run failed: $(head -c 1200 "$LAST_ERR")"
        return
    fi
    # Positive controls: the probe ran IN the sandbox and can observe it.
    expect_line "positive control: the probe received its request" \
        '{"check":"stdin","subject":"request","result":"ok"}' "$out"
    expect_line "positive control: its own binary is visible at /pnc/pledge_compile_api" \
        '{"check":"self","subject":"/pnc/pledge_compile_api","result":"present"}' "$out"
    if [ "$PROBE_MODE" = "unit" ]; then
        # Under the unit, RestrictAddressFamilies= refuses IP sockets
        # outright (on top of bwrap's empty network namespace); AF_UNIX
        # still works (the unix probe below gets "not_found", not this).
        expect_line "IP sockets refused by the unit (loopback)" \
            '{"check":"tcp-control","subject":"127.0.0.1:9","result":"af_refused"}' "$out"
        expect_line "IP sockets refused by the unit (outside)" \
            '{"check":"tcp","subject":"1.1.1.1:443","result":"af_refused"}' "$out"
    else
        expect_line "positive control: loopback connect is refused (networking works at all)" \
            '{"check":"tcp-control","subject":"127.0.0.1:9","result":"refused"}' "$out"
        expect_line "no network: an outside address is unreachable" \
            '{"check":"tcp","subject":"1.1.1.1:443","result":"network_unreachable"}' "$out"
    fi
    for p in "${RUN_PATHS[@]}"; do
        expect_line "$p is not in the run sandbox" "{\"check\":\"path\",\"subject\":\"$p\",\"result\":\"not_found\"}" "$out"
    done
    expect_line "the builder socket cannot be connected to" \
        "{\"check\":\"unix\",\"subject\":\"$SOCKET\",\"result\":\"not_found\"}" "$out"
    expect_line "runs as the builder uid ($BUILDER_UID)" \
        "{\"check\":\"uid\",\"subject\":\"self\",\"result\":\"$BUILDER_UID\"}" "$out"
    for v in "cpu|30/30" "nofile|64/64" "fsize|0/0" "core|0/0"; do
        expect_line "rlimit ${v%%|*} = ${v#*|}" \
            "{\"check\":\"rlimit\",\"subject\":\"${v%%|*}\",\"result\":\"${v#*|}\"}" "$out"
    done
    v="$(sed -n 's/.*"check":"pid","subject":"self","result":"\([0-9]*\)".*/\1/p' <<<"$out")"
    if [ -n "$v" ] && [ "$v" -le 3 ]; then
        ok "$PASS_LABEL run phase: own PID namespace (probe is pid $v)"
    else
        fail "$PASS_LABEL run phase: probe pid '$v': not in a fresh PID namespace"
    fi
    v="$(sed -n 's/.*"check":"env","subject":"keys","result":"\([^"]*\)".*/\1/p' <<<"$out")"
    # bwrap itself sets PWD (for --chdir /).
    case "$v" in
        LD_LIBRARY_PATH,PWD|LD_LIBRARY_PATH,PWD,RUST_LOG) ok "$PASS_LABEL run phase: cleared environment ($v)" ;;
        *) fail "$PASS_LABEL run phase: unexpected environment keys '$v' (canary or PNC_ leaked?)" ;;
    esac
    if grep -qF "$CANARY" <<<"$out"; then fail "$PASS_LABEL run phase: the environment canary is visible"; fi
    # The whole filesystem, by name: the binary and its shared libraries,
    # nothing else.
    v="$(sed -n 's/.*"check":"tree","subject":"\/","result":"\([^"]*\)".*/\1/p' <<<"$out")"
    local f bad="" seen_exe=0
    IFS=',' read -r -a files <<<"$v"
    for f in "${files[@]}"; do
        case "$f" in
            /pnc/pledge_compile_api) seen_exe=1 ;;
            */ld-linux*.so*|*/libc.so.*|*/libm.so.*|*/libgcc_s.so.*) ;;
            *) bad="$bad $f" ;;
        esac
    done
    if [ "$seen_exe" -eq 1 ] && [ -z "$bad" ] && [ "${#files[@]}" -le 8 ]; then
        ok "$PASS_LABEL run phase: the sandbox's whole filesystem is the binary and its libraries (${#files[@]} files: $v)"
    else
        fail "$PASS_LABEL run phase: unexpected files in the run sandbox:${bad:- (binary missing or too many files)} [$v]"
    fi
}

# [unit] only: what the unit itself (no bwrap) can see, so that "absent in
# the sandbox" can be attributed to bwrap, not to ProtectProc=/Temporary
# FileSystem=. Each must be visible to a plain process under the unit.
unit_controls() {
    local p
    for p in /proc/sys/kernel/overflowuid "$SOCKET" "$BIN" "$TOOLCHAIN_DIR"; do
        if timeout 60 systemd-run --quiet --wait --pipe --collect --unit="pnc-verify-ctl-$$-$RANDOM" \
            "${UNIT_PROPS[@]}" -p RuntimeMaxSec=30 /usr/bin/test -e "$p" </dev/null >/dev/null 2>&1; then
            ok "[unit] control: $p IS visible to the unit without bwrap"
        else
            fail "[unit] control: $p is not visible to the unit itself; its sandbox probe would prove nothing"
        fi
    done
}

if [ "$VERIFY_PROBES" = "runuser" ] || [ "$VERIFY_PROBES" = "both" ]; then
    PROBE_MODE="runuser"; PASS_LABEL="[runuser]"
    sandbox_probes
    run_phase_probes
fi
if [ "$VERIFY_PROBES" = "unit" ] || [ "$VERIFY_PROBES" = "both" ]; then
    PROBE_MODE="unit"; PASS_LABEL="[unit]"
    load_unit_props
    if compare_unit_props; then
        unit_controls
        sandbox_probes
        run_phase_probes
    else
        fail "[unit] probes skipped: the probe service would not match the running builder"
    fi
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
refuses "sandbox-verify-run with a token" "must hold no secrets" \
    PNC_AUTH_TOKENS="$TOK" "${BENV[@]}" "$BIN" sandbox-verify-run

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
