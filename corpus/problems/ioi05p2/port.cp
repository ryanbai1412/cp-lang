// Source: IOI 2005, Mountains, https://dmoj.ca/problem/ioi05p2
// Problem: a track of n <= 1e9 rails with slopes d_i, all 0 at first, then up to 1e5 commands
//   ending in E: "I a b D" sets d_a..=d_b to D; "Q h" prints how many rails a car starting at
//   height 0 passes before the track would rise above h (heights stay in [0, 1e9]).
// Solution: read every command first and compress rails to the cut points a and b + 1. A lazy
//   SegTree over the pieces keeps (total rise, max prefix rise) under slope assignment. A ride
//   binary-searches the first piece whose prefix max exceeds h, then divides the height left
//   by that piece's slope for the rails it covers. O(q log^2 q).
// original: corpus/submissions/5032753.cpp

let n: Int = read()
var tags: List<Int> = []
var ca: List<Int> = []
var cb: List<Int> = []
var cd: List<Int> = []

while true:
    let tok: Str = read()
    if tok[0] == 'E': break
    if tok[0] == 'I':
        let (a, b, d): (Int, Int, Int) = read()
        tags.push(0)
        ca.push(a)
        cb.push(b)
        cd.push(d)
    else:
        let h: Int = read()
        tags.push(1)
        ca.push(h)
        cb.push(0)
        cd.push(0)

struct SegVal:
    total: Int
    max_pre: Int
    len: Int

let rail_monoid = Monoid<SegVal>(
    id: SegVal(0, 0, 0),
    op: (x, y) => SegVal(x.total + y.total, max(x.max_pre, x.total + y.max_pre), x.len + y.len),
)

let assign_slope = Action<SegVal, ?Int>(
    id: None,
    compose: (f, g) => if f == None: g else: f,
    apply: (f, x, len) => if f == None: x else: SegVal(f! * x.len, max(0, f! * x.len), x.len),
    over: rail_monoid,
)

var pts = [1, n + 1]
for i in 0..tags.len:
    if tags[i] == 0:
        pts.push(ca[i])
        pts.push(cb[i] + 1)
pts = pts.sorted()
var xs: List<Int> = []
for x in pts:
    if xs.len == 0 or xs.last! != x: xs.push(x)

fn coord(x: Int) -> Int:
    let lo = xs.first_ge(x) ?? xs.len
    lo

domain Seg = 0..(xs.len - 1)
let init = [Seg] i => SegVal(0, 0, xs[i.to_idx + 1] - xs[i.to_idx])
var st = SegTree(assign_slope.over, init, act: assign_slope)

for i in 0..tags.len:
    if tags[i] == 0:
        let l = Seg(coord(ca[i]))
        let r = Seg(coord(cb[i] + 1) - 1)
        st.apply(l..=r, cd[i])
    else:
        let h = ca[i]
        // the first segment where the car's height would pass h
        if let seg_i = Seg.first_true(s => st.fold(..=s).max_pre > h):
            let lo = seg_i.to_idx
            let cur = if lo == 0: 0 else: st.fold(..seg_i).total
            let node = st.get(seg_i)
            let slope = node.total / node.len
            let extra = (h - cur) / slope
            print(xs[lo] - 1 + extra)
        else:
            print(n)
