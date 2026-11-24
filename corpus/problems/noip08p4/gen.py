import sys, random
# seed size: a permutation of 1..n (n <= 1000); half the cases are built by a random two-stack
# run, so they are sortable
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 1000 if mx else int(sys.argv[2])
n = 1000 if mx else r.randint(1, min(1000, S + 1))
if mx or r.random() < .5:
    # read the input backwards from a sorted output: undo random pops and pushes
    out, st, inp = list(range(n, 0, -1)), [[], []], []
    while out or st[0] or st[1]:
        k = r.randint(0, 1)
        if out and (r.random() < .5 or not (st[0] or st[1])):
            x = out.pop()
            if not st[k] or st[k][-1] > x: st[k].append(x)
            else: st[1 - k].append(x) if not st[1 - k] or st[1 - k][-1] > x else out.append(x)
            if out and out[-1] == x: break
        elif st[k]: inp.append(st[k].pop())
        elif st[1 - k]: inp.append(st[1 - k].pop())
    p = list(range(1, n + 1)); r.shuffle(p)
    if len(inp) == n: p = inp[::-1]
else:
    p = list(range(1, n + 1)); r.shuffle(p)
print(n); print(*p)
