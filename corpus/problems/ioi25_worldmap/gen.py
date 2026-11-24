# IOI 2025 World Map, sample-grader stdin: T <= 50 scenarios, each `N M` then M pairs `A B`
# with 1 <= A < B <= N <= 40, distinct pairs, the graph connected (a map exists exactly then).
# gen.py SEED SIZE: N <= min(SIZE, 40); shapes follow the subtasks: path, tree, complete,
# country 1 adjacent to all, small N, and random density. gen.py --max: 50 scenarios at N = 40.
import sys, random


def scenario(r, n, shape):
    order = list(range(1, n + 1))
    r.shuffle(order)
    if shape == "path":
        edges = {(i, i + 1) for i in range(1, n)}
    else:
        edges = {tuple(sorted((order[i], order[r.randrange(i)]))) for i in range(1, n)}
    if shape == "complete":
        edges = {(a, b) for a in range(1, n + 1) for b in range(a + 1, n + 1)}
    elif shape == "hub":
        edges = {(1, b) for b in range(2, n + 1)}
    if shape in ("hub", "random", "deep"):
        p = r.choice([0.05, 0.2, 0.5, 0.9])
        edges |= {(a, b) for a in range(1, n + 1) for b in range(a + 1, n + 1) if r.random() < p}
    if shape == "deep":  # a long path plus chords: deep DFS trees, many back edges
        edges |= {tuple(sorted((order[i - 1], order[i]))) for i in range(1, n)}
    edges = sorted(edges)
    r.shuffle(edges)
    return f"{n} {len(edges)}\n" + "".join(f"{a} {b}\n" for a, b in edges)


SHAPES = ["path", "tree", "complete", "hub", "random", "deep"]
if sys.argv[1] == "--max":
    r = random.Random(0)
    print(50)
    print("".join(scenario(r, 40, SHAPES[t % len(SHAPES)]) for t in range(50)), end="")
    sys.exit()
r = random.Random(int(sys.argv[1]))
cap = max(1, min(40, int(sys.argv[2])))
t = r.randint(1, 50)
print(t)
print("".join(scenario(r, r.randint(1, cap), r.choice(SHAPES)) for _ in range(t)), end="")
