#!/usr/bin/env bash
# The one way CI installs a tool, run by .github/actions/install-tools on a
# cache miss: each pinned name@version is its project's prebuilt release for
# this runner, fetched through retry.sh into <dir>/bin. A release download
# takes seconds where `cargo install` built nextest for 3-4 minutes.
#   install-tools.sh DIR name@version...   install into DIR/bin
#   install-tools.sh --url name@version OS ARCH   print the release URL
set -euo pipefail

# name@version OS ARCH (GitHub's runner.os and runner.arch) -> "URL BINARY"
release() {
    local name=${1%@*} v=${1#*@} os=$2 arch=$3 target ext=tar.gz exe=""
    case $os-$arch in
        Linux-X64) target=x86_64-unknown-linux-gnu ;;
        macOS-ARM64) target=aarch64-apple-darwin ;;
        macOS-X64) target=x86_64-apple-darwin ;;
        Windows-X64) target=x86_64-pc-windows-msvc ext=zip exe=.exe ;;
        *) echo "install-tools: no runner target for $os-$arch" >&2; return 1 ;;
    esac
    case $name in
        just)
            [ "$os" = Linux ] && target=x86_64-unknown-linux-musl
            echo "https://github.com/casey/just/releases/download/$v/just-$v-$target.$ext just$exe" ;;
        cargo-nextest)
            [ "$os" = macOS ] && target=universal-apple-darwin
            echo "https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-$v/cargo-nextest-$v-$target.$ext cargo-nextest$exe" ;;
        cargo-llvm-cov)
            echo "https://github.com/taiki-e/cargo-llvm-cov/releases/download/v$v/cargo-llvm-cov-$target.$ext cargo-llvm-cov$exe" ;;
        cargo-mutants)
            # No arm64 macOS build; Rosetta runs the x86_64 one.
            [ "$os" = macOS ] && target=x86_64-apple-darwin
            echo "https://github.com/sourcefrog/cargo-mutants/releases/download/v$v/cargo-mutants-$target.$ext cargo-mutants$exe" ;;
        actionlint)
            local plat
            case $os-$arch in
                Linux-X64) plat=linux_amd64 ;; macOS-ARM64) plat=darwin_arm64 ;;
                macOS-X64) plat=darwin_amd64 ;; Windows-X64) plat=windows_amd64 ;;
            esac
            echo "https://github.com/rhysd/actionlint/releases/download/v$v/actionlint_${v}_$plat.$ext actionlint$exe" ;;
        *) echo "install-tools: no release recipe for $name; add one to scripts/install-tools.sh" >&2; return 1 ;;
    esac
}

if [ "${1:-}" = --url ]; then
    release "$2" "$3" "$4"
    exit
fi

# shellcheck source=scripts/retry.sh
. "$(dirname "$0")/retry.sh"
dir=$1
shift
mkdir -p "$dir/bin"
for tool in "$@"; do
    read -r url bin < <(release "$tool" "$RUNNER_OS" "$RUNNER_ARCH")
    tmp=$(mktemp -d)
    archive="$tmp/${url##*/}"
    fetch -o "$archive" "$url"
    case $archive in
        *.zip) unzip -q -o "$archive" -d "$tmp/x" 2>/dev/null || 7z x -y -o"$tmp/x" "$archive" >/dev/null ;;
        *) mkdir "$tmp/x" && tar -xzf "$archive" -C "$tmp/x" ;;
    esac
    found=$(find "$tmp/x" -type f -name "$bin" | head -n 1)
    [ -n "$found" ] || { echo "install-tools: $url has no $bin" >&2; exit 1; }
    mv "$found" "$dir/bin/$bin"
    chmod +x "$dir/bin/$bin"
    rm -rf "$tmp"
    echo "install-tools: $tool from $url"
done
