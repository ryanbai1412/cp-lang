// A view of a temporary keeps the temporary alive for as long as the view
// is used: held in a name, a loop, a tuple, an optional, a list, a field.
fn g(xs: List<Int>) -> Int:
    var t = 0
    for r in xs.sorted().runs(): t += r.len * r.first!
    t

fn f(n: Int) -> Int:
    let w = [n, n + 7, 1].sorted().windows(2)
    if n == 0: return w[1].sum()
    f(n - 1) + w[1].sum()

let a = [3, 1, 3, 2, 2, 3]
for (r, run) in a.sorted(by: desc).runs().items:
    print(r, run.len, run.first!)
let rs = a.sorted().runs()
let w = a.sorted().windows(2)
print(rs.len, rs[0].len, w.len, w[0].sum())
let p = Some(a.sorted().windows(2))
let q = (a.sorted().runs(), a.sorted(by: desc).windows(2))
print(p!.len, p![1].sum(), q.0.len, q.1[0].sum())
var ws = [a.windows(2)]
for k in 1..=3:
    ws.push([k * 5, 2, k].sorted().windows(2))
print([v[1].sum() for v in ws])
var h = (a.windows(2), 0)
h.0 = a.sorted(by: desc).windows(3)
print(h.0[0].sum(), h.0[3].sum())
print(g([2, 2, 5]), f(3), [r.len for r in [5, 4, 4, 1].sorted().runs()])
