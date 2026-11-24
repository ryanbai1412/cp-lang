#!/usr/bin/env bash
# The real clipboard round trip (handbook/domains/tools.md, Submitting; ci/R29):
# `cpl b -c` with each clipboard tool this OS's cpl uses, alone on PATH, then
# the OS's paste tool must give back a.cpp byte for byte (CRLF read as LF on
# Windows). a.cpp itself must end its lines LF. The source has a
# UTF-8 comment, which the C++ keeps. macOS: pbcopy. Windows: clip. Linux:
# xclip and xsel on an Xvfb display, wl-copy on a headless sway (needs xvfb,
# xclip, xsel, wl-clipboard, sway). WSL's clip.exe is only stubbed, in
# crates/cpl-cli/tests/clipboard.rs. CI: ci.yml `cli`.
set -euo pipefail
cpl="$(cd "$(dirname "${1:-target/debug/cpl}")" && pwd)/$(basename "${1:-target/debug/cpl}")"
dir="$(mktemp -d)"
pids=()
cleanup() { set +e; kill ${pids[@]+"${pids[@]}"} 2>/dev/null; rm -rf "$dir"; }
trap cleanup EXIT
printf '// naïve → ok\nlet n: Int = read()\nprint(n * 2)\n' >"$dir/a.cp"

fail() { echo "clipboard-roundtrip: $*" >&2; exit 1; }

# roundtrip TOOL PATH PASTE...: copy with TOOL found on PATH, paste into $dir/pasted.
roundtrip() {
    local tool=$1 path=$2; shift 2
    rm -f "$dir/a.cpp" "$dir/pasted"
    (cd "$dir" && env -u CPL_CLIPBOARD PATH="$path" "$cpl" b a.cp -c) 2>"$dir/err" || { cat "$dir/err" >&2; fail "$tool: cpl b -c failed"; }
    grep -q "with \`$tool" "$dir/err" || { cat "$dir/err" >&2; fail "$tool: cpl did not copy with $tool"; }
    ! grep -q $'\r' "$dir/a.cpp" || fail "$tool: cpl wrote a CR into a.cpp; its lines must end LF"
    "$@" || fail "$tool: nothing to paste back"
    # Windows' clipboard hands text back CRLF; the text is the same.
    if [ -n "${crlf:-}" ]; then sed -i 's/\r$//' "$dir/pasted"; fi
    cmp "$dir/a.cpp" "$dir/pasted" || {
        od -c "$dir/a.cpp" | head -5 >&2; echo "pasted:" >&2; od -c "$dir/pasted" | head -5 >&2
        fail "$tool: the clipboard is not a.cpp"
    }
    echo "clipboard-roundtrip: $tool ok ($(wc -c <"$dir/a.cpp") bytes)"
}

# alone TOOL...: a PATH holding only these tools.
alone() {
    local bin="$dir/bin-$1"
    mkdir -p "$bin"
    for t in "$@"; do ln -sf "$(command -v "$t")" "$bin/$t"; done
    echo "$bin"
}

case "$(uname -s)" in
Darwin)
    roundtrip pbcopy "$(alone pbcopy)" sh -c 'LC_ALL=en_US.UTF-8 pbpaste >"$0/pasted"' "$dir"
    ;;
MINGW* | MSYS* | CYGWIN*)
    # PowerShell writes the file itself: its stdout would re-encode the text.
    win="$(cygpath -w "$dir/pasted")"
    crlf=1
    # Windows' own PATH, so `clip` is clip.exe and not a Git Bash tool.
    roundtrip clip "$(cygpath -u "$SYSTEMROOT")/System32" powershell -NoProfile -Command \
        "[IO.File]::WriteAllText('$win', (Get-Clipboard -Raw), (New-Object Text.UTF8Encoding \$false))"
    ;;
Linux)
    export XDG_RUNTIME_DIR="$dir/xdg"
    mkdir -m 700 "$XDG_RUNTIME_DIR"
    unset WAYLAND_DISPLAY
    # Each server says it is ready on a pipe: Xvfb writes its display number,
    # sway runs the config's exec.
    mkfifo "$dir/x-ready" "$dir/sway-ready"
    Xvfb -displayfd 3 -nolisten tcp 3>"$dir/x-ready" >"$dir/xvfb.log" 2>&1 & pids+=($!)
    read -r -t 20 n <"$dir/x-ready" || { cat "$dir/xvfb.log" >&2; fail "Xvfb did not start"; }
    export DISPLAY=:$n
    roundtrip xclip "$(alone xclip)" sh -c 'xclip -o -selection clipboard >"$0/pasted"' "$dir"
    roundtrip xsel "$(alone xsel)" sh -c 'xsel --clipboard --output >"$0/pasted"' "$dir"
    unset DISPLAY
    echo "exec sh -c 'echo \$WAYLAND_DISPLAY >$dir/sway-ready'" >"$dir/sway.conf"
    WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 WLR_RENDERER=pixman \
        sway -c "$dir/sway.conf" >"$dir/sway.log" 2>&1 & pids+=($!)
    read -r -t 20 wayland <"$dir/sway-ready" || { cat "$dir/sway.log" >&2; fail "sway did not start"; }
    export WAYLAND_DISPLAY=$wayland
    # wl-copy runs cat to read its input.
    roundtrip wl-copy "$(alone wl-copy cat)" sh -c 'wl-paste --no-newline >"$0/pasted"' "$dir"
    ;;
*) fail "no clipboard round trip for $(uname -s)" ;;
esac
