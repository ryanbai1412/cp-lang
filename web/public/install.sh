#!/bin/sh
# Installs CP Lang: curl -fsSL https://cp-lang.org/install.sh | sh
#
#   sh install.sh [--channel stable|edge] [--version 0.3.0 | edge-YYYY-MM-DD]
#                 [--no-modify-path]
#
# stable (the default) is the newest v* release, beta while 0.x; until the
# first one exists it installs edge, the build of the newest green main.
#
# Downloads the release for this platform from GitHub, checks it against the
# sha256 in the release's latest.json, unpacks it into
# $CPL_HOME/toolchains/<toolchain>/ (CPL_HOME defaults to ~/.cpl) and puts the
# launcher at $CPL_HOME/bin/cpl. CPL_RELEASES_URL points it at a mirror.
set -eu

releases="${CPL_RELEASES_URL:-https://github.com/ryanbai1412/cp-lang/releases}"
releases="${releases%/}"
home="${CPL_HOME:-$HOME/.cpl}"
channel=stable
version=""
modify_path=1

say() { printf 'cpl-install: %s\n' "$*"; }
die() { printf 'cpl-install: error: %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
    case "$1" in
        --channel) [ $# -ge 2 ] || die "--channel needs stable or edge"; channel="$2"; shift ;;
        --channel=*) channel="${1#--channel=}" ;;
        --edge) channel=edge ;;
        --stable) channel=stable ;;
        --version) [ $# -ge 2 ] || die "--version needs a value"; version="$2"; shift ;;
        --version=*) version="${1#--version=}" ;;
        --no-modify-path) modify_path=0 ;;
        -h | --help) sed -n '2,10s/^# \{0,1\}//p' "$0" 2>/dev/null || true; exit 0 ;;
        *) die "unknown option: $1" ;;
    esac
    shift
done
case "$channel" in
    stable | edge) ;;
    *) die "--channel takes stable or edge, not $channel" ;;
esac

case "$(uname -s)" in
    Linux) os=unknown-linux-musl ;;
    Darwin) os=apple-darwin ;;
    MINGW* | MSYS* | CYGWIN*) die "on Windows, run in PowerShell: irm https://cp-lang.org/install.ps1 | iex" ;;
    *) die "no prebuilt CP Lang for $(uname -s); build from source: cargo install --path crates/cpl-cli" ;;
esac
case "$(uname -m)" in
    x86_64 | amd64) arch=x86_64 ;;
    aarch64 | arm64) arch=aarch64 ;;
    *) die "no prebuilt CP Lang for $(uname -m)" ;;
esac
if [ "$os" = apple-darwin ] && [ "$arch" = x86_64 ] && [ "$(sysctl -n sysctl.proc_translated 2>/dev/null || echo 0)" = 1 ]; then
    arch=aarch64
fi
target="$arch-$os"

if [ -n "$version" ]; then
    case "$version" in
        edge | edge-[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]) tag="$version" ;;
        stable) tag="" ;;
        v[0-9]*) tag="$version" ;;
        [0-9]*.[0-9]*.[0-9]*) tag="v$version" ;;
        *) die "--version takes 0.3.0, edge or edge-YYYY-MM-DD, not $version" ;;
    esac
elif [ "$channel" = edge ]; then
    tag=edge
else
    tag=""
fi
if [ -n "$tag" ]; then
    base="$releases/download/$tag"
else
    base="$releases/latest/download"
fi

if command -v curl >/dev/null 2>&1; then
    fetch() { curl -fsSL --retry 2 -o "$2" "$1"; }
elif command -v wget >/dev/null 2>&1; then
    fetch() { wget -q -O "$2" "$1"; }
else
    die "needs curl or wget"
fi
if command -v sha256sum >/dev/null 2>&1; then
    sha256() { sha256sum "$1" | cut -d' ' -f1; }
elif command -v shasum >/dev/null 2>&1; then
    sha256() { shasum -a 256 "$1" | cut -d' ' -f1; }
else
    die "needs sha256sum or shasum to check the download"
fi
command -v tar >/dev/null 2>&1 || die "needs tar"

mkdir -p "$home/tmp"
tmp="$(mktemp -d "$home/tmp/install.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT INT TERM

say "fetching $base/latest.json"
if ! fetch "$base/latest.json" "$tmp/latest.json"; then
    # Until the first stable release, the stable channel installs edge.
    [ -z "$tag" ] || die "could not download $base/latest.json"
    base="$releases/download/edge"
    say "no stable release yet; fetching $base/latest.json"
    fetch "$base/latest.json" "$tmp/latest.json" || die "could not download $base/latest.json"
fi
manifest="$(tr -d '\n\r' <"$tmp/latest.json")"
field() { printf '%s' "$1" | sed -n "s/.*\"$2\": *\"\\([^\"]*\\)\".*/\\1/p"; }
toolchain="$(field "$manifest" toolchain)"
case "$toolchain" in
    "" | .* | */*) die "the release manifest names no valid toolchain" ;;
esac
entry="$(printf '%s' "$manifest" | sed -n "s/.*\"$target\": *{\\([^}]*\\)}.*/\\1/p")"
file="$(field "$entry" file)"
want="$(field "$entry" sha256)"
if [ -z "$file" ] || [ -z "$want" ]; then
    die "$toolchain has no build for $target"
fi
case "$file" in
    */* | .*) die "the release manifest names a bad file: $file" ;;
esac

dest="$home/toolchains/$toolchain"
if [ -x "$dest/bin/cpl" ]; then
    say "$toolchain is already installed"
else
    say "downloading $toolchain for $target"
    fetch "$base/$file" "$tmp/$file" || die "could not download $base/$file"
    got="$(sha256 "$tmp/$file")"
    [ "$got" = "$want" ] || die "$file does not match its checksum (expected $want, got $got); nothing was installed"
    mkdir -p "$tmp/unpack" "$home/toolchains"
    tar -xzf "$tmp/$file" -C "$tmp/unpack"
    [ -x "$tmp/unpack/bin/cpl" ] || die "$file has no bin/cpl"
    cp "$tmp/latest.json" "$tmp/unpack/release.json"
    rm -rf "$dest"
    mv "$tmp/unpack" "$dest"
fi

mkdir -p "$home/bin"
cp "$dest/bin/cpl" "$home/bin/cpl.new"
chmod 755 "$home/bin/cpl.new"
mv -f "$home/bin/cpl.new" "$home/bin/cpl"

settings="$home/settings.toml"
default="${version:-$channel}"
key="$channel"
case "$default" in
    stable | edge) ;;
    *) key="" ;;
esac
{
    if [ -f "$settings" ]; then
        grep -vE "^(default|${key:-default}) *=" "$settings" || true
    fi
    if [ "$default" != stable ]; then
        printf 'default = "%s"\n' "$default"
    fi
    if [ -n "$key" ]; then
        printf '%s = "%s"\n' "$key" "$toolchain"
    fi
} >"$tmp/settings.toml"
mv "$tmp/settings.toml" "$settings"

say "installed $toolchain into $dest"
say "launcher: $home/bin/cpl"

case ":${PATH}:" in
    *":$home/bin:"*) say "run: cpl --version"; exit 0 ;;
esac
line="export PATH=\"$home/bin:\$PATH\""
if [ "$modify_path" = 1 ]; then
    for rc in "$HOME/.profile" "$HOME/.bashrc" "$HOME/.zshrc"; do
        case "$rc" in
            */.bashrc | */.zshrc) [ -f "$rc" ] || continue ;;
        esac
        if ! grep -qsF "$line" "$rc"; then
            printf '\n%s\n' "$line" >>"$rc"
            say "added $home/bin to PATH in $rc"
        fi
    done
    if [ -d "$HOME/.config/fish" ]; then
        mkdir -p "$HOME/.config/fish/conf.d"
        printf 'fish_add_path "%s/bin"\n' "$home" >"$HOME/.config/fish/conf.d/cpl.fish"
        say "added $home/bin to PATH in ~/.config/fish/conf.d/cpl.fish"
    fi
fi
say "open a new terminal, or run this now, then: cpl --version"
printf '\n    %s\n\n' "$line"
