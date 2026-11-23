// ioi05p2 Mountain: rails 1..=n start flat; commands set the slope of a block of rails or ask how
// many rails a car with energy h passes before the height first exceeds h. Offline: the command
// endpoints cut the track into pieces of one slope each, keyed by the rails' cut ordinals so a
// piece's left end is the count of rails before it. A segment tree over the pieces keeps (rise,
// highest prefix, length) under "assign slope", and a ride walks right while the highest prefix
// stays at most h, then divides the height left by the slope of the piece where it breaks.
let n = read<1..=1_000_000_000>()
domain Rail = 1..=n
enum Cmd:
    @tag("I") Set(a: Rail, b: Rail, d: -1_000_000_000..=1_000_000_000)
    @tag("Q") Ask(h: 0..=1_000_000_000)
    @tag("E") End
let cmds = read_all<Cmd>(until: c => c == Cmd.End)
let (domain P, piece) = (0..Rail.count).pieces(
    cuts: [Int(x) for c in cmds if let Cmd.Set(a, b, _) = c for x in [a.l, b.r]],
)
type Rise = (rise: Int, peak: Int, len: Int)
let track = Monoid<Rise>(
    id: (rise: 0, peak: 0, len: 0),
    op: (x, y) => (rise: x.rise + y.rise, peak: max(x.peak, x.rise + y.peak), len: x.len + y.len),
)
let slope = Action<Rise, ?Int>(
    id: None,
    compose: (new, old) => new ?? old,
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
        Set(a, b, d) => t.apply(piece.covering(Int(a.l)..Int(b.r)), Some(d))
        End => continue
        Ask(h) =>
            let stop = t.right_while(P.cuts.first!, x => x.peak <= h)
            if stop == P.cuts.last!:
                print(n)
            else:
                let p = stop.r!
                let seg = t.get(p)
                let base = t.fold(..stop).rise
                print(piece[p].l + (h - base) / (seg.rise / seg.len))
