// IOI 2005 Mountains: n rails, each with a slope (all 0 at first). `I a b d` sets rails a..=b to
// slope d; `Q h` asks how many rails a car with energy h passes before the height would exceed h.
// Read everything first, cut the rails into pieces at every update boundary, and keep a segment
// tree of (length, total rise, highest prefix) with "set slope" as the action.
enum Cmd: Set(a: Int, b: Int, d: Int) = 'I', Ask(h: Int) = 'Q', End = 'E'
domain Rail = 1..=read Int
let cmds = read stream Cmd until End
let (domain Piece, piece) = Rail.pieces(cuts: cmds.flat_map(c => match c:
    Set(a, b, _): [Rail(a).l, Rail(b).r]
    _: []))

struct Rise(len: Int, total: Int, peak: Int)
let then = (x: Rise, y: Rise) => Rise(x.len + y.len, x.total + y.total, max(x.peak, x.total + y.peak))
let slope = (d: Int, x: Rise) => Rise(x.len, d * x.len, max(0, d * x.len))
var track = SegTree(then, [Piece] p => Rise(piece[p].len, 0, 0), act: set(slope))

for cmd in cmds:
    match cmd:
        Set(a, b, d): track.apply(Piece.covering(Rail(a)..=Rail(b)), d)
        Ask(h):
            let (end, before) = track.max_right(Piece.first!, ok: r => r.peak <= h)
            let rails = before.len + match end.r:   // the piece where the car stops
                Some(p): (h - before.total) / max(1, track.slope_at(p))
                None: 0
            print(rails)
