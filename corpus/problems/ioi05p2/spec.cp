// ioi05p2 Mountain: rails 1..=n start flat; commands set the slope of a block of rails or ask how
// far a car with energy h gets (the last rail before the height first exceeds h). Offline: the
// command endpoints cut the track into stretches of one slope each; a segment tree over them
// keeps (rise, highest prefix, length) under "assign slope", and the query walks right while the
// highest prefix stays at most h, then divides inside the stretch where it breaks.
let n = read<1..=1_000_000_000>()
domain Rail = 1..=n
enum Cmd:
    @tag("I") Set(a: Rail, b: Rail, d: -1_000_000_000..=1_000_000_000)
    @tag("Q") Ask(h: 0..=1_000_000_000)
    @tag("E") End
let cmds = read_all<Cmd>(until: c => c == Cmd.End)
let (domain P, piece) = Rail.pieces(
    cuts: cmds.flat_map(c => if let Cmd.Set(a, b, _) = c: [a.l, b.r] else: [])
)
type Rise = (rise: Int, peak: Int, len: Int)
let track = Monoid<Rise>(
    id: (rise: 0, peak: 0, len: 0),
    op: (x, y) => (rise: x.rise + y.rise, peak: max(x.peak, x.rise + y.peak), len: x.len + y.len),
)
let slope = Action<Rise, ?Int>(
    id: None,
    compose: (old, new) => new ?? old,
    apply: (d, x, _) => if let Some(v) = d: (
        rise: v * x.len,
        peak: max(0, v * x.len),
        len: x.len,
    ) else: x,
    over: track,
)
var t = SegTree(track, [P] p => (rise: 0, peak: 0, len: piece[p].len), act: slope)
for c in cmds:
    match c:
        Set(a, b, d) => t.apply(piece.covering(a..=b), Some(d))
        End => continue
        Ask(h) =>
            let stop = t.right_while(P.cuts.first!, x => x.peak <= h)
            if stop == P.cuts.last!:
                print(n)
            else:
                let p = stop.r!
                let seg = t.get(p)
                let base = t.fold(..stop).rise
                print(Int(piece[p].first!.l) + (h - base) / (seg.rise / seg.len))
