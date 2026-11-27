// expect: ok, output "3 0 1 2 6"
// rule: R04.4.30
// A type parameter the body indexes a table or an array by is a domain parameter, as one it calls
// a `Domain` fn on is: `[D] d => e` and `var s: [D]Int` need no written `D: Domain`.
fn ords<D>(xs: List<D>) -> List<Int>:
    let at = [D] d => d.ord
    var seen: [D]Int = 0
    for x in xs: seen[x] += at[x]
    [seen[x] for x in xs]
fn width<D>(x: D) -> Int:
    let t = [D] d => 1
    t.sum()
domain V = 0..6
let o = ords([V(3), V(0), V(1), V(2)])
print(o[0], o[1], o[2], o[3], width(V(0)))
