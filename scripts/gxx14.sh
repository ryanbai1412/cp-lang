#!/bin/bash
# Prints the path of a g++ 14 for corpus references, fetching conda-forge's
# into ~/.cache/cpl/gcc14 once (about 2 minutes) when the machine has none:
# Ubuntu 22.04's apt stops at g++ 12, and hosted runners ship g++ 14.
set -euo pipefail
dir="${CPL_GXX_HOME:-$HOME/.cache/cpl/gcc14}"
gxx="$dir/bin/x86_64-conda-linux-gnu-g++"
if [ ! -x "$gxx" ]; then
    [ "$(uname -sm)" = "Linux x86_64" ] ||
        { echo "gxx14: only linux x86_64 is fetched; install g++ 14 as g++-14" >&2; exit 1; }
    . "$(dirname "$0")/retry.sh"
    mm="$dir.mamba"
    mkdir -p "$mm"
    echo "gxx14: fetching g++ 14 from conda-forge into $dir (once)" >&2
    [ -x "$mm/bin/micromamba" ] ||
        retry sh -c "curl -fsSL https://micro.mamba.pm/api/micromamba/linux-64/latest | tar -xj -C '$mm' bin/micromamba"
    retry "$mm/bin/micromamba" create -y -q -r "$mm/root" -p "$dir" -c conda-forge gxx_linux-64=14 >&2
fi
echo "$gxx"
