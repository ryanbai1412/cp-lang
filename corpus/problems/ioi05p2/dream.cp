// IOI 2005 Mountains: n rails, each with a slope (all 0 at first). `I a b d` sets rails a..=b to
// slope d; `Q h` asks how many rails a car with energy h passes before the height would exceed h.
// Read everything first, cut the rails into pieces at every update boundary, and keep a segment
// tree of (total rise, highest prefix, length) with "set slope" as the action.
let n: 1..=1e9 = read()
domain Rail = 1..=n
enum Cmd: Set(a: Rail, b: Rail, d: -1e9..=1e9) | Ask(h: 0..=1e9)
var cmds = List<Cmd>()
while true:
    match read<Char>():
        'E' => break
        'I' =>
            let (a, b, d): (Rail, Rail, -1e9..=1e9) = read()
            cmds.push(Cmd.Set(a, b, d))
        _ => cmds.push(Cmd.Ask(read()))
let (domain Piece, piece) = Rail.range.pieces(
    cuts: [x for c in cmds if let Cmd.Set(a, b, _) = c for x in [a.l, b.r]]
)

struct Rise:
    total: Int
    peak: Int
    len: Int
let then = Monoid<Rise>(
    id: Rise(0, 0, 0),
    op: (x, y) => Rise(x.total + y.total, max(x.peak, x.total + y.peak), x.len + y.len),
)
let slope = Action<Rise, ?Int>(
    id: None,
    compose: (new, old) => new ?? old,
    over: then,
    apply: (d, x, _) => if let s = d: Rise(s * x.len, max(0, s * x.len), x.len) else: x,
)
var track = SegTree(then, [Piece] p => Rise(0, 0, piece[p].len), act: slope)

for c in cmds:
    match c:
        Set(a, b, d) => track.apply(piece.covering(a..=b), d)
        Ask(h) =>
            let stop = track.right_while(Piece.cuts.first!, x => x.peak <= h)
            if let p = stop.r:  // the piece where the car stops
                let at = track.get(p)
                print(Int(piece[p].l) + (h - track.fold(..stop).total) / (at.total / at.len))
            else:
                print(n)
