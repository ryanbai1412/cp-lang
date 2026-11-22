# p19 judge in the protocol `cpl --interactor` uses: run as `PROG <input>`,
# talking to the solution over its own stdin/stdout. Exit 0 iff the secret
# is found in at most 30 queries. Same rules as research/dryrun/p19_guess_interact.py,
# which instead expects to spawn the solution from argv[2:].
import sys
n, secret = map(int, open(sys.argv[1]).read().split())
print(n, flush=True)
for q in range(31):
    line = sys.stdin.readline().split()
    if not line:
        sys.exit(f"no output after {q} queries")
    if line[0] == "!":
        sys.exit(0 if int(line[1]) == secret else f"wrong answer {line[1]}")
    if q == 30:
        sys.exit("more than 30 queries")
    print(">=" if secret >= int(line[1]) else "<", flush=True)
