// A step along a domain is a position, not an optional, where a test or a loop
// orders the name it steps from against another position of the domain:
// `l + 1` inside `if l < r`, after `if l >= r: return`, and for `m` in `l..r`.
// Where the optional is asked for (`!`, `??`) it is still one.
let n = 5
domain I = 0..n
let a: [I]Int = [I] i => i.to_idx
fn span(l: I, r: I) -> Int:
    if l >= r: return a[l]
    a[l] + span(l.next!, r)
fn back(l: I, r: I) -> Int:
    if l < r: a[r] + back(l, r.prev!) else: a[l]
print(span(I(0), I(4)), back(I(1), I(3)))
var dp: [I, I]Int = 0
for l in I.rev:
    for r in l..:
        if l == r: continue
        dp[l, r] = [dp[l, m] + dp[m.next!, r] for m in l..r].min()! + a[r]
print(dp[I(0), I(4)])
var i = I(0)
let last = I(4)
var steps = 0
while i < last:
    steps += a[i.next!]
    i = i.next!
print(steps, (i.prev ?? i).ord)
