let n: 2..=6 = read()
let es: [n - 1](a: 1..=9, b: 1..=9) = read()
print(sum(e.a for e in es), sorted(es, by: .b)[0].a)
var m: HashMap<Int, Int> = [3: 4, 5: 6]
m[3] += 1
print(m[3], m[5], m.len)
let plain = [1: 2]
print(plain[1])
domain G = 1..=n
fn two(x: G.cuts, y: G.cuts) -> Int:
    let g = x.r else: return 0
    let h = y.r else: return 0
    g.ord + h.ord + 1
fn two2(x: G.cuts, y: G.cuts) -> Int:
    let g = x.r and let h = y.r else: return -1
    g.ord * 10 + h.ord
print(two(G.first!.l, G.last!.l), two(G.last!.r, G.first!.l), two2(G.first!.r, G.last!.l), two2(G.first!.l, G.last!.r))
let d: [G]?Int = [G] i => if i.ord % 2 == 0: None else: i.ord
fn arrival(i: G) -> ?Int: i.ord * 100 + d[i]?
fn both(i: G, j: G) -> ?Int:
    let s = d[i]? + d[j]?
    s * 2
print(arrival(G(1)) ?? -1, arrival(G(2)) ?? -1, both(G(2), G(4)) ?? -1, both(G(2), G(3)) ?? -1)
let no: ?Int = None
let yes: ?Int = 4
print(min(7, no, yes), min(3, no), max(no, 2, yes), min(no, yes) ?? -1, max(no, no) ?? -1)
print(min(n * n, min(x * x for x in 3..=n), min(y for y in 1..=n / 2)))
