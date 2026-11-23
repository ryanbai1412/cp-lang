#!/usr/bin/env bash
# The CLI half of the acceptance run's `install`, `test` and `submit` steps on one OS,
# following web/content/docs/guides/getting-started.mdx word for word. Prints a JSON
# object: {"os": ..., "checks": [{"name", "pass", "output"}]}. Run by
# .github/workflows/acceptance.yml after the site's installer.
set -u
os="${1:-$(uname -s)}"
repo="$(cd "$(dirname "$0")/../.." && pwd)"
work="$(mktemp -d)"
cd "$work" || exit 1
checks=()

esc() { python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()[-1500:]))'; }

check() {  # check NAME EXPECTED-SUBSTRING-OR-EMPTY COMMAND... (stdin passes through)
  local name="$1" want="$2"; shift 2
  local out code pass
  out="$("$@" 2>&1)"; code=$?
  pass=false
  if [ "$code" -eq 0 ] && { [ -z "$want" ] || printf '%s' "$out" | grep -qF -- "$want"; }; then pass=true; fi
  checks+=("{\"name\": \"$name\", \"pass\": $pass, \"output\": $(printf '%s' "$out" | esc)}")
}

cat > sum.cp <<'CP'
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()
print(a.sum())
CP
printf 'let n: 1..=2e5 = read()\r\ndomain V = 1..=n\r\nlet a: [V]Int = read()\r\nprint(a.sum())\r\n' > crlf.cp
printf '5\n3 1 4 1 5\n' > in.txt
printf '5\r\n3 1 4 1 5\r\n' > in-crlf.txt

check version "" cpl --version
check check "" cpl check sum.cp
check run 14 sh -c 'cpl run sum.cp < in.txt'
check build "" cpl build sum.cp -o sum.cpp
check gxx "" g++ -std=c++17 -O2 sum.cpp -o sum
check native 14 sh -c './sum < in.txt'
check size "" sh -c 'test "$(wc -c < sum.cpp)" -lt 65536 && wc -c < sum.cpp'
check crlf-source 14 sh -c 'cpl run crlf.cp < in.txt'
check crlf-input 14 sh -c 'cpl run sum.cp < in-crlf.txt'
check fmt "" cpl fmt sum.cp
check lsp "" python3 "$repo/editors/lsp-smoke.py" acceptance cpl lsp
check setup-neovim "" cpl setup neovim --dir "$work/nvim/pack/cpl/start/cpl"
check setup-vim "" cpl setup vim --dir "$work/vim/pack/cpl/start/cpl"
check jetbrains-community "loads in Community" python3 - <<'PY'
import io, re, urllib.request, zipfile
url = "https://github.com/ryanbai1412/cp-lang/releases/download/edge/cp-lang-jetbrains.zip"
outer = zipfile.ZipFile(io.BytesIO(urllib.request.urlopen(url, timeout=60).read()))
xml = None
for name in outer.namelist():
    if name.endswith(".jar"):
        jar = zipfile.ZipFile(io.BytesIO(outer.read(name)))
        if "META-INF/plugin.xml" in jar.namelist():
            xml = jar.read("META-INF/plugin.xml").decode()
assert xml, "no META-INF/plugin.xml in the edge JetBrains plugin"
deps = re.findall(r"<depends[^>]*>([^<]+)</depends>", xml)
hard = [d for d in deps if "ultimate" in d.lower()]
assert not hard, f"plugin requires {hard}, so IntelliJ IDEA Community refuses it"
print("loads in Community: depends on", deps)
PY
check vscode-vsix "extension/package.json" python3 - <<'PY'
import io, urllib.request, zipfile
url = "https://github.com/ryanbai1412/cp-lang/releases/download/edge/cp-lang.vsix"
print(zipfile.ZipFile(io.BytesIO(urllib.request.urlopen(url, timeout=60).read())).namelist())
PY

printf '{"os": "%s", "checks": [' "$os"
( IFS=,; printf '%s' "${checks[*]}" )
printf ']}\n'
