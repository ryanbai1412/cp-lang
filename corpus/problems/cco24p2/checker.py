"""cco24p2 checker: `checker.py input output answer`. -1 exactly when the answer is; otherwise
the answer's K, then each pizza's stack and each person's stack in 1..=K, such that laying the
pizzas in order and letting the people take in order always finds their flavour on top."""
import sys

inp = list(map(int, open(sys.argv[1]).read().split()))
out = open(sys.argv[2]).read().split()
ans = open(sys.argv[3]).read().split()
n = inp[0]
a, b = inp[1:1 + n], inp[1 + n:1 + 2 * n]
if ans[:1] == ["-1"]:
    sys.exit(0 if out == ["-1"] else 1)
if len(out) != 1 + 2 * n or not all(t.isdigit() for t in out) or out[0] != ans[0]:
    sys.exit(1)
k = int(out[0])
c, d = [int(t) for t in out[1:1 + n]], [int(t) for t in out[1 + n:]]
if not all(1 <= s <= k for s in c + d):
    sys.exit(1)
stacks = [[] for _ in range(k + 1)]
for s, f in zip(c, a):
    stacks[s].append(f)
for s, f in zip(d, b):
    if not stacks[s] or stacks[s].pop() != f:
        sys.exit(1)
sys.exit(0)
