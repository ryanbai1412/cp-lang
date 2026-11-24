// expect: ok, output "YES"
// rule: 05.2.1 05.3.5
// A position read from a list inside a `[P]List<P>` is a copied value, so it can be stored past its block.
domain P = 1..=3
var q: [P]List<P> = [P] p => [p]
var last: [P]?P = None
for p in P:
    let t = q[p][0]
    last[p] = t
print(yes(last[P(1)] == Some(P(1))))
