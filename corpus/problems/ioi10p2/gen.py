"""gen.py SEED SIZE | --max: IOI 2010 Hotter Colder, Q then Q games `N secret`. A seed plays
up to 50 * SIZE games with N up to 500 (subtasks 1-3) or up to 500 000 000 (subtask 4),
secrets anywhere or at the ends. --max plays 2000 games at N = 500 000 000: the statement
allows 1 000 000 calls, more than a Python judge answers in the 60 s budget, so it keeps
the largest N and both ends of the range."""
import random, sys

if sys.argv[1] == "--max":
    r = random.Random(2010)
    q, big = 2000, True
else:
    r = random.Random(int(sys.argv[1]))
    q, big = r.randint(1, 50 * int(sys.argv[2])), r.random() < 0.5
top = 500_000_000 if big else 500
games = []
for i in range(q):
    n = top if sys.argv[1] == "--max" else r.randint(1, top)
    s = [1, n, (n + 1) // 2, r.randint(1, n)][i % 4]
    games.append(f"{n} {s}")
print(q)
print("\n".join(games))
