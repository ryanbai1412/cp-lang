#!/usr/bin/env bash
# The CLI half of the acceptance run's `install`, `test` and `submit` steps on one OS,
# following web/content/docs/guides/getting-started.mdx word for word. Prints a JSON
# object: {"os": ..., "checks": [{"name", "pass", "output"}]}. Run by
# .github/workflows/acceptance.yml after the site's installer.
set -u
os="${1:-$(uname -s)}"
work="$(mktemp -d)"
cd "$work" || exit 1
checks=()

esc() { python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()[-1500:]))'; }

check() {  # check NAME EXPECTED-SUBSTRING-OR-EMPTY COMMAND...
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
check lsp-starts capabilities python3 -c '
import subprocess
body = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}"
p = subprocess.Popen(["cpl", "lsp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
p.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
p.stdin.flush()
try:
    out = p.stdout.read1(4096) if hasattr(p.stdout, "read1") else p.stdout.read(200)
finally:
    p.kill()
print(out.decode(errors="replace"))
'

printf '{"os": "%s", "checks": [' "$os"
( IFS=,; printf '%s' "${checks[*]}" )
printf ']}\n'
