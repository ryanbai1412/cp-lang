#!/usr/bin/env bash
# Emacs for the fresh-home setup test (crates/cpl-cli/tests/golden/setup.rs)
# and the Emacs smoke test, on the CI runner's OS: apt on Linux, Homebrew on
# macOS, Chocolatey on Windows.
set -euo pipefail
. "$(dirname "$0")/retry.sh"
case "$(uname -s)" in
    Linux)
        # man-db's trigger alone can take minutes on a hosted runner.
        sudo rm -f /var/lib/man-db/auto-update
        retry timeout 180 sudo apt-get install -y -q --no-install-recommends -o DPkg::Lock::Timeout=60 emacs-nox >/dev/null
        ;;
    Darwin) retry brew install --quiet emacs >/dev/null ;;
    MINGW* | MSYS* | CYGWIN*) retry choco install emacs -y --no-progress >/dev/null ;;
    *) echo "install-emacs: no installer for $(uname -s)" >&2; exit 1 ;;
esac
emacs --version | head -1
