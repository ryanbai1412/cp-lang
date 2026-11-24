// Source: IOI 2005, Mountains, https://dmoj.ca/problem/ioi05p2
// limits: 1400 ms, 256 MB
// Problem: a track of 1 <= n <= 1e9 rails with slopes d_i, all 0 at first, then up to 1e5
//   commands ending in E: "I a b D" sets d_a..=d_b to D (|D| <= 1e9); "Q h" (0 <= h <= 1e9)
//   prints how many rails a car starting at height 0 passes before the track would rise above h
//   (heights stay in [0, 1e9]).
// Solution: read every command first; their endpoints cut the rails into pieces of one slope
//   each. A lazy SegTree over the pieces keeps (rise, highest prefix, length) under "assign
//   slope". A ride walks right while the highest prefix stays at most h, then divides the
//   height left by the slope of the piece where it stops. O(q log q).
// original: corpus/submissions/5032753.cpp
// Bounds: n: 1..=1e9; a, b: 1..=n; d: -1e9..=1e9

let n: 1..=1e9 = read()
domain Rail = 1..=n
enum Cmd: Set(a: Rail, b: Rail, d: -1e9..=1e9) | Ask(h: 0..=1e9)

var cmds = List<Cmd>()
while true:
    let tag: Char = read()
    if tag == 'E': break
    if tag == 'I':
        let (a, b, d): (Rail, Rail, -1e9..=1e9) = read()
        cmds.push(Cmd.Set(a, b, d))
    else:
        cmds.push(Cmd.Ask(read()))

let (domain Piece, piece) = Rail.range.pieces(
    cuts: [x for c in cmds if let Cmd.Set(a, b, _) = c for x in [a.l, b.r]],
)

struct Rise:
    rise: Int
    peak: Int
    len: Int

let then = Monoid<Rise>(
    id: Rise(0, 0, 0),
    op: (x, y) => Rise(x.rise + y.rise, max(x.peak, x.rise + y.peak), x.len + y.len),
)
let assign = Action<Rise, ?Int>(
    id: None,
    compose: (new, old) => new ?? old,
    apply: (d, x, _) => if let v = d: Rise(v * x.len, max(0, v * x.len), x.len) else: x,
    over: then,
)
var track = SegTree(then, [Piece] p => Rise(0, 0, piece[p].len), act: assign)

for c in cmds:
    match c:
        Set(a, b, d): track.apply(piece.covering(a..=b), d)
        Ask(h):
            let stop = track.right_while(Piece.cuts.first!, x => x.peak <= h)
            if let p = stop.r:
                let at = track.get(p)
                let before = track.fold(..stop).rise
                print(Int(piece[p].l) + (h - before) / (at.rise / at.len))
            else:
                print(n)
