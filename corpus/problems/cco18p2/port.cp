
const N = 100
domain V = 0..N

// The two cost-1 chains 0-1-2-5-6-9-10-... and 0-3-4-7-8-11-...
fn cheap(r: V, c: V) -> Bool:
    if r.ord < 2: return c.ord <= 3 and (r.ord, c.ord) != (1, 3)
    if r.ord % 2 == 0: c.ord == r.ord + 3
    else: c.ord == r.ord + 1 and r.ord + 2 < N

print(N)
for r in V.all.drop_last(1):
    print([if cheap(r, c): 1 else: 100 for c in V if c > r])
