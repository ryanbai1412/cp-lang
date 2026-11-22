# p19 judge for `n secret`; exit 0 iff the solution finds secret in <= 30 queries.
# `PROG input SOLUTION...` spawns the solution; `PROG input` alone talks to it
# over its own stdin/stdout, the protocol `cpl --interactor` uses.
import subprocess, sys
n, secret = map(int, open(sys.argv[1]).read().split())
if sys.argv[2:]:
    p = subprocess.Popen(sys.argv[2:], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    fin, fout = p.stdout, p.stdin
else:
    fin, fout = sys.stdin, sys.stdout
def say(s):
    fout.write(s + "\n"); fout.flush()
say(str(n))
for q in range(31):
    line = fin.readline().split()
    if not line: sys.exit(f"no output after {q} queries")
    if line[0] == "!":
        sys.exit(0 if int(line[1]) == secret else f"wrong answer {line[1]}")
    if q == 30: sys.exit("more than 30 queries")
    say(">=" if secret >= int(line[1]) else "<")
