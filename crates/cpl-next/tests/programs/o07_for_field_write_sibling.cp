// expect: ok, output "3"
// (spec) R05.8.1
// The everyday shape: walk one field, update a sibling field. No generator of the user's own.
struct G:
    adj: List<Int>
    seen: List<Int>
fn go() -> Int:
    var g = G([0, 1, 2], [0, 0, 0])
    for v in g.adj: g.seen[v] = 1
    g.seen[0] + g.seen[1] + g.seen[2]
print(go())
