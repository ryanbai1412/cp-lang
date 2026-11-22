# Runs a solution binary against the judge for `n secret`; exit 0 iff it finds secret in <= 30 queries.
import subprocess, sys
n, secret = map(int, open(sys.argv[1]).read().split())
p = subprocess.Popen(sys.argv[2:], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
p.stdin.write(f"{n}\n"); p.stdin.flush()
for q in range(31):
    line = p.stdout.readline().split()
    if not line: sys.exit(f"no output after {q} queries")
    if line[0] == "!":
        sys.exit(0 if int(line[1]) == secret else f"wrong answer {line[1]}")
    if q == 30: sys.exit("more than 30 queries")
    p.stdin.write(">=\n" if secret >= int(line[1]) else "<\n"); p.stdin.flush()
