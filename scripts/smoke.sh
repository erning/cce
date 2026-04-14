#!/usr/bin/env bash
# scripts/smoke.sh — manual end-to-end test runner for cce.sh.
#
# Run before committing any change to cce.sh. Exits 0 if all tests pass,
# 1 otherwise. Self-contained: builds its own temp config dir, does not
# touch ~/.config/cce, and does not depend on `claude` being installed —
# tests use /bin/echo and /bin/sh as the target command.
#
# Bash 3.2 compatible (matches the runtime requirement of cce.sh itself).

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CCE="$SCRIPT_DIR/../cce.sh"

if [[ ! -x "$CCE" ]]; then
  echo "Error: cce.sh not found or not executable at $CCE" >&2
  exit 1
fi

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

passed=0
failed=0
fail_names=()

if [[ -t 1 ]]; then
  C_PASS=$'\033[32m'
  C_FAIL=$'\033[31m'
  C_SKIP=$'\033[33m'
  C_OFF=$'\033[0m'
else
  C_PASS=""
  C_FAIL=""
  C_SKIP=""
  C_OFF=""
fi

# --- helpers -----------------------------------------------------------------

section() {
  printf '\n=== %s ===\n' "$1"
}

pass() {
  passed=$((passed + 1))
  printf '  %s✓%s %s\n' "$C_PASS" "$C_OFF" "$1"
}

fail() {
  failed=$((failed + 1))
  fail_names+=("$1")
  printf '  %s✗%s %s\n' "$C_FAIL" "$C_OFF" "$1"
  if [[ -n "${2:-}" ]]; then
    printf '    %s\n' "$2"
  fi
}

skip() {
  printf '  %s-%s %s (skipped: %s)\n' "$C_SKIP" "$C_OFF" "$1" "$2"
}

# Run cce capturing stdout/stderr/exit_code into globals. PATH is sanitized
# so fzf is never picked up; the no-name path always takes the list branch.
run_cce() {
  local out="$TMP/_out" err="$TMP/_err"
  PATH=/bin:/usr/bin "$CCE" "$@" >"$out" 2>"$err"
  exit_code=$?
  stdout=$(cat "$out")
  stderr=$(cat "$err")
}

expect_exit() {
  if [[ "$exit_code" == "$1" ]]; then
    pass "$2"
  else
    fail "$2" "exit=$exit_code expected=$1; stderr=$stderr"
  fi
}

expect_stdout_has() {
  if [[ "$stdout" == *"$1"* ]]; then
    pass "$2"
  else
    fail "$2" "stdout did not contain: $1"
  fi
}

expect_stderr_has() {
  if [[ "$stderr" == *"$1"* ]]; then
    pass "$2"
  else
    fail "$2" "stderr did not contain: $1"
  fi
}

expect_stdout_eq() {
  if [[ "$stdout" == "$1" ]]; then
    pass "$2"
  else
    fail "$2" "stdout was '$stdout', expected '$1'"
  fi
}

# --- fixtures ----------------------------------------------------------------

CONF="$TMP/conf/cce"
mkdir -p "$CONF"

cat >"$CONF/probe.env" <<'EOF'
export ANTHROPIC_AUTH_TOKEN=fake
export PROBE=hello-from-env
EOF

# Bad-name files exercising the discovery whitelist
echo 'export ANTHROPIC_AUTH_TOKEN=fake' >"$CONF/bad name.env"
echo 'export ANTHROPIC_AUTH_TOKEN=fake' >"$CONF/-dash.env"
echo 'export ANTHROPIC_AUTH_TOKEN=fake' >"$CONF/.hidden.env"

# Bash parse error (unclosed `$(`)
# shellcheck disable=SC2016  # the literal `$(` is what we want
echo 'this is not valid bash $(' >"$CONF/bork.env"

# Runtime command failure inside an env file
cat >"$CONF/runtime_fail.env" <<'EOF'
export ANTHROPIC_AUTH_TOKEN=fake
false
EOF

chmod 600 "$CONF"/*.env "$CONF"/.hidden.env

# Empty config dir (the cce/ subdir exists but contains no .env)
EMPTY="$TMP/empty/cce"
mkdir -p "$EMPTY"

# Config root whose cce/ subdir does NOT exist yet
NOCONF="$TMP/noconf"
mkdir -p "$NOCONF"

export XDG_CONFIG_HOME="$TMP/conf"

# --- tests -------------------------------------------------------------------

section "Basics"

run_cce --version
expect_exit 0 "--version exits 0"
expect_stdout_has "cce" "--version mentions cce"

run_cce --help
expect_exit 0 "--help exits 0"
expect_stdout_has "Usage:" "--help shows usage line"

section "Argument parsing — error paths"

run_cce --typo
expect_exit 1 "unknown long flag rejected"
expect_stderr_has "unknown option" "unknown long flag error message"

run_cce -x glm
expect_exit 1 "unknown short flag rejected"
expect_stderr_has "unknown option" "unknown short flag error message"

run_cce -- glm
expect_exit 1 "'--' before NAME rejected"
expect_stderr_has "must come after the environment name" "'--' before NAME error message"

run_cce --command "" probe
expect_exit 1 "--command '' rejected"
expect_stderr_has "non-empty argument" "--command '' error message"

section "NAME-then-flag passthrough"

run_cce -c /bin/echo probe --help
expect_exit 0 "cce <name> --help runs the target command"
expect_stdout_eq "--help" "--help is forwarded verbatim, not parsed by cce"

run_cce -c /bin/echo probe -- --help
expect_exit 0 "'-- ' after NAME is back-compat-accepted"
expect_stdout_eq "--help" "back-compat: '--' is consumed, --help still forwarded"

run_cce -c /bin/echo probe one two three
expect_exit 0 "multi-token passthrough exits 0"
expect_stdout_eq "one two three" "multi-token passthrough preserves order"

# shellcheck disable=SC2016  # the inner $vars are expanded by /bin/sh, not by us
run_cce -c /bin/sh probe -c 'echo "TOKEN=$ANTHROPIC_AUTH_TOKEN PROBE=$PROBE"'
expect_exit 0 "source + exec runs end-to-end"
expect_stdout_eq "TOKEN=fake PROBE=hello-from-env" "exported env vars reach the exec'd child"

section "XDG / HOME resolution"

XDG_CONFIG_HOME=relative run_cce nonexistent
expect_exit 1 "relative XDG_CONFIG_HOME falls back"
expect_stderr_has "not an absolute path" "relative XDG warning"

XDG_CONFIG_HOME="" run_cce nonexistent
expect_exit 1 "empty XDG_CONFIG_HOME falls back to HOME silently"
if [[ "$stderr" != *"not an absolute path"* ]]; then
  pass "empty XDG fallback is silent (no spurious warning)"
else
  fail "empty XDG fallback is silent" "stderr=$stderr"
fi

# `env -i` strips both HOME and XDG_CONFIG_HOME — neither path is usable.
out=$(env -i PATH=/bin:/usr/bin "$CCE" probe 2>&1)
rc=$?
if [[ $rc -eq 1 && "$out" == *"HOME is not set"* ]]; then
  pass "no HOME and no XDG_CONFIG_HOME errors out cleanly"
else
  fail "no HOME and no XDG_CONFIG_HOME errors out cleanly" "exit=$rc out=$out"
fi
unset out rc

section "Env name whitelist (CLI input)"

run_cce .hidden
expect_exit 1 "leading-dot name rejected"
expect_stderr_has "invalid environment name" "leading-dot error message"

run_cce ../etc/passwd
expect_exit 1 "path-traversal name rejected"

run_cce 'bad name'
expect_exit 1 "name with space rejected"

section "Discovery filter (no-fzf list path)"

run_cce
expect_exit 0 "list mode exits 0"
expect_stdout_has "  probe" "list shows the valid env"
expect_stderr_has "skipping invalid env file name: 'bad name.env'" "warns on 'bad name.env'"
expect_stderr_has "skipping invalid env file name: '-dash.env'" "warns on '-dash.env'"

# Each warning should appear exactly once (the dedup fix).
bad_count=$(printf '%s\n' "$stderr" | grep -c "bad name.env" || true)
if [[ "$bad_count" == "1" ]]; then
  pass "discovery warnings are not duplicated"
else
  fail "discovery warnings are not duplicated" "saw $bad_count occurrences of 'bad name.env'"
fi

# .hidden.env is dot-prefixed; bash glob skips it entirely, so neither stream mentions it.
if [[ "$stdout" != *".hidden"* && "$stderr" != *".hidden"* ]]; then
  pass ".hidden.env is silently ignored (bash glob skip)"
else
  fail ".hidden.env is silently ignored" "appeared in output"
fi

XDG_CONFIG_HOME="$TMP/empty" run_cce
expect_exit 0 "empty config dir exits 0"
expect_stdout_has "No environments found" "empty config message"

XDG_CONFIG_HOME="$NOCONF" run_cce
expect_exit 0 "missing config subdir exits 0"
expect_stdout_has "Directory does not exist yet" "missing config dir message"

section "Source error paths"

run_cce -c /bin/echo bork
expect_exit 1 "bash parse error in env file exits 1"
expect_stderr_has "syntax error in environment file" "parse error has cce context"

run_cce -c /bin/echo runtime_fail
expect_exit 1 "runtime command failure inside env file exits 1"
expect_stderr_has "failed while loading environment file" "runtime failure has ERR-trap context"

section "Command resolution"

run_cce -c /no/such/binary probe
expect_exit 127 "missing target command exits 127"
expect_stderr_has "command not found" "missing command error message"

run_cce -c /bin/echo nope
expect_exit 1 "missing env file exits 1"
expect_stderr_has "environment file not found" "missing env file error message"

section "Permission warning"

chmod 666 "$CONF/probe.env"
run_cce -c /bin/echo probe
expect_exit 0 "666 perms still runs"
expect_stderr_has "writable by group or other" "666 triggers warning"

chmod 660 "$CONF/probe.env"
run_cce -c /bin/echo probe
expect_exit 0 "660 perms still runs"
expect_stderr_has "writable by group or other" "660 triggers warning"

chmod 600 "$CONF/probe.env"
run_cce -c /bin/echo probe
expect_exit 0 "600 perms still runs"
if [[ "$stderr" != *"writable by group"* ]]; then
  pass "600 produces no permission warning"
else
  fail "600 produces no permission warning" "stderr=$stderr"
fi

section "Internal variable protection (readonly)"

# An env file that tries to override cce's internal _CCE_COMMAND must abort
# the source via readonly + the ERR trap, surfacing a clear context.
cat >"$CONF/ro_override.env" <<'EOF'
export ANTHROPIC_AUTH_TOKEN=fake
_CCE_COMMAND=/bin/true
EOF
chmod 600 "$CONF/ro_override.env"

run_cce -c /bin/echo ro_override
expect_exit 1 "override of _CCE_COMMAND aborts run"
expect_stderr_has "readonly variable" "readonly error surfaces"
# Note: bash's readonly-assignment error exits the sourced file via a
# different path than a failed command, so the ERR trap's "failed while
# loading" context line is not guaranteed to fire here. The bare
# "readonly variable" message from bash itself is already unambiguous.

# Same deal for _CCE_ARGS — the exec-time array must stay frozen.
cat >"$CONF/ro_args.env" <<'EOF'
export ANTHROPIC_AUTH_TOKEN=fake
_CCE_ARGS=(wrong)
EOF
chmod 600 "$CONF/ro_args.env"

run_cce -c /bin/echo ro_args
expect_exit 1 "override of _CCE_ARGS aborts run"
expect_stderr_has "readonly variable" "_CCE_ARGS readonly error surfaces"

# Regression for the rename itself: a plain `ENV_NAME=production` in a user
# env file must *not* collide with any cce internal, because the internal is
# now prefixed. This is the whole point of the _CCE_ rename.
cat >"$CONF/nocollide.env" <<'EOF'
export ANTHROPIC_AUTH_TOKEN=fake
ENV_NAME=production
COMMAND=my-command
ENV_DIR=/tmp/elsewhere
export PROBE_ENV_NAME="$ENV_NAME"
EOF
chmod 600 "$CONF/nocollide.env"

# shellcheck disable=SC2016  # the inner $var is expanded by /bin/sh, not us
run_cce -c /bin/sh nocollide -c 'echo "$PROBE_ENV_NAME"'
expect_exit 0 "plain ENV_NAME/COMMAND/ENV_DIR in env file do not collide"
expect_stdout_eq "production" "unprefixed names are free for user use"

section "Non-TTY fzf downgrade"

# Stand up a stub `fzf` on PATH that would scream if invoked. If the TTY
# guard works, stdin </dev/null should route the no-name path to list mode
# without ever touching the stub. Without the guard, the stub would run and
# appear in stderr.
FZF_STUB_DIR="$TMP/fzfstub"
mkdir -p "$FZF_STUB_DIR"
cat >"$FZF_STUB_DIR/fzf" <<'EOF'
#!/bin/sh
echo "STUB_FZF_WAS_CALLED" >&2
exit 1
EOF
chmod +x "$FZF_STUB_DIR/fzf"

# Dedicated config root with exactly one valid env, so the picker branch
# *would* otherwise trigger.
SOLO_ROOT="$TMP/solo"
SOLO="$SOLO_ROOT/cce"
mkdir -p "$SOLO"
cat >"$SOLO/only.env" <<'EOF'
export ANTHROPIC_AUTH_TOKEN=fake
EOF
chmod 600 "$SOLO/only.env"

tty_out=$(PATH="$FZF_STUB_DIR:/bin:/usr/bin" XDG_CONFIG_HOME="$SOLO_ROOT" "$CCE" </dev/null 2>"$TMP/_err")
tty_rc=$?
tty_err=$(cat "$TMP/_err")

if [[ $tty_rc -eq 0 ]]; then
  pass "non-TTY stdin with fzf on PATH exits 0"
else
  fail "non-TTY stdin with fzf on PATH exits 0" "rc=$tty_rc err=$tty_err"
fi
if [[ "$tty_err" != *"STUB_FZF_WAS_CALLED"* ]]; then
  pass "non-TTY stdin does not invoke fzf (TTY guard)"
else
  fail "non-TTY stdin does not invoke fzf (TTY guard)" "stub was called: $tty_err"
fi
if [[ "$tty_out" == *"  only"* ]]; then
  pass "non-TTY stdin downgrades to list output"
else
  fail "non-TTY stdin downgrades to list output" "stdout=$tty_out"
fi
unset tty_out tty_rc tty_err

section "Empty ARGS exec (set -u workaround)"

run_cce -c /bin/echo probe
expect_exit 0 "empty ARGS exec exits 0"
expect_stdout_eq "" "empty ARGS produces empty stdout (no unbound-var crash)"

section "Lint baseline"

if bash -n "$CCE" 2>"$TMP/_lint"; then
  pass "bash -n cce.sh"
else
  fail "bash -n cce.sh" "$(cat "$TMP/_lint")"
fi

if command -v shellcheck >/dev/null 2>&1; then
  if shellcheck -s bash "$CCE" >"$TMP/_lint" 2>&1; then
    pass "shellcheck -s bash cce.sh"
  else
    fail "shellcheck -s bash cce.sh" "$(cat "$TMP/_lint")"
  fi
else
  skip "shellcheck -s bash cce.sh" "shellcheck not installed"
fi

if command -v shfmt >/dev/null 2>&1; then
  if shfmt -d -i 2 -ci "$CCE" >"$TMP/_lint" 2>&1; then
    pass "shfmt -d -i 2 -ci cce.sh"
  else
    fail "shfmt -d -i 2 -ci cce.sh" "diff present (run 'shfmt -w -i 2 -ci cce.sh')"
  fi
else
  skip "shfmt -d -i 2 -ci cce.sh" "shfmt not installed"
fi

# --- summary -----------------------------------------------------------------

echo
total=$((passed + failed))
if [[ $failed -eq 0 ]]; then
  printf '%s%d/%d passed%s\n' "$C_PASS" "$passed" "$total" "$C_OFF"
  exit 0
else
  printf '%s%d/%d passed, %d failed%s\n' "$C_FAIL" "$passed" "$total" "$failed" "$C_OFF"
  for n in "${fail_names[@]}"; do
    printf '  - %s\n' "$n"
  done
  exit 1
fi
