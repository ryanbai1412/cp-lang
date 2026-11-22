// ioi05p2 Mountain: rails 1..=n start flat; commands set the slope of a block of rails or ask how
// far a car with energy h gets (the last rail before the height first exceeds h). Offline: the
// command endpoints cut the track into stretches of one slope each; a segment tree over them
// keeps (rise, highest prefix, length) under "assign slope", and the query walks right while the
// highest prefix stays at most h, then divides inside the stretch where it breaks.
let n = read<1..=1_000_000_000>()
enum Cmd:
    @tag("I") Set(a: 1..=n, b: 1..=n, d: -1_000_000_000..=1_000_000_000)
    @tag("Q") Ask(h: 0..=1_000_000_000)
    @tag("E") End
var cmds: List<Cmd> = []
var cuts = [1, n + 1]
while true:
    let c = read<Cmd>()
    if c == Cmd.End:
        break
    if c is Cmd.Set(a, b, _):
        cuts.push(a)
        cuts.push(b + 1)
    cmds.push(c)
let (domain S, starts) = compress(cuts)
type Rail = (rise: Int, peak: Int, len: Int)
let track = Monoid<Rail>(id: (rise: 0, peak: 0, len: 0),
    op: (x, y) => (rise: x.rise + y.rise, peak: max(x.peak, x.rise + y.peak), len: x.len + y.len))
let slope = Action<Rail, ?Int>(id: none, compose: (old, new) => new ?? old,
    apply: (d, x, _) => if d is some(v): (rise: v * x.len, peak: max(0, v * x.len), len: x.len) else: x,
    over: track)
let lens = [S] s => if s == S.last!: 0 else: starts.get(s.next!) - starts.get(s)
var t = SegTree(track, [S] s => (rise: 0, peak: 0, len: lens[s]), act: slope)
for c in cmds:
    if c is Cmd.Set(a, b, d):
        t.apply(starts.at(a)!..starts.at(b + 1)!, some(d))
    elif c is Cmd.Ask(h):
        let stop = t.right_while(S.cuts.first!, x => x.peak <= h)
        if stop == S.cuts.last!:
            print(n)
        else:
            let s = stop.r!
            let seg = t.get(s)
            let base = t.fold(..stop).rise
            print(starts.get(s) - 1 + (h - base) / (seg.rise / seg.len))
