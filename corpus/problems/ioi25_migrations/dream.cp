// Full score needs symbols ≤ 4 and at most 8 messages (N = 10 000). Keep two
// candidate lists for the diameter's ends, P and Q, which start as all sites.
// Five phases shrink them. At a phase's first site, split P and Q into blocks:
// the current ends lie in one block of P and one of Q, and that block pair is
// sent as one signal over the phase's next k sites (silence is pair 0; otherwise
// which site speaks and its symbol 1..=4 spell the index). Sites that arrive
// during the phase may still become ends, so they join both lists. After five
// phases each list has at most 4 sites, and each of the last three sites sends a
// symbol that narrows the end pairs still possible: the editorial's case tables,
// written once and shared by both sides. Site N-3 sees the ends among the 24 pairs of
// `last3` (P × Q and pairs with itself), and its symbol is which group of five holds
// them; both runs then keep the lists `after3` gives that symbol. Site N-2 does the same
// with the ten pairs of `last2` in groups of two, leaving three sites a, b, c, and site
// N-1 names one of `last1`'s five pairs outright. Each table pairs the speaking site
// with the others because it may itself have just become an end. Lists shorter than 4
// are padded with `None`, which no end equals, so a padded entry is never spoken.
// limits: 3000 ms, 2048 MB
use std.math.{iroot}

domain Site = 0..10_000

struct Phase(start: Int, m: Int, b: Int, k: Int, first: Bool):
    @get fn block(self) -> Int = self.m.ceil_div(self.b)
    @get fn pairs(self) -> List<(Int, Int)> =
        [(x, y) for x in 0..self.b for y in (if self.first: x else: 0)..self.b]

fn phase_plan(n: Int) -> List<Phase>:
    for start in 4..n - 3:
        var phases: List<Phase> = []
        var (m, at) = (start - 1, start - 1)
        while at < n - 3 and phases.len < 5:
            let first = phases.is_empty
            let pairs = (b: Int) => if first: b * (b + 1) / 2 else: b * b
            let b = (1..=iroot(8 * (m + 1), 3) + 2).argmin(
                b => (m + 1).ceil_div(b) + (pairs(b) - 1).ceil_div(4)
            )!
            let k = (pairs(b) - 1).ceil_div(4)
            phases.push(Phase(at, m, b, k, first))
            (m, at) = (m.ceil_div(b) + k - 1, at + k)
        if at == n - 3 and phases.len == 5 and m <= 4: return phases
    assert false, "no start fits five phases"

fn signal(c: Int, from: Int) -> ?(Int, Int) =
    if c > 0: Some((from + (c - 1) / 4, (c - 1) % 4 + 1)) else: None
fn heard(s: List<0..=4>, window: Range<Int>) -> Int =
    window.find(j => s[j] != 0).map(j => 4 * (j - window.start) + s[j]) ?? 0

fn block(xs: List<Site>, x: Int, size: Int) -> List<Site> = xs.drop(x * size).take(size).collect()
fn padded(xs: List<Site>) -> List<?Site> = [Some(x) for x in xs].resized(4, fill: None)

fn last3(p: List<?Site>, q: List<?Site>, me: ?Site) -> List<(?Site, ?Site)> = [
    (p[0], q[0]), (p[0], q[1]), (p[0], q[2]), (p[1], q[0]), (p[1], q[1]),
    (p[0], q[3]), (p[0], me), (p[1], q[2]), (p[1], q[3]), (p[1], me),
    (p[2], q[0]), (p[2], q[1]), (p[3], q[0]), (p[3], q[1]), (me, q[0]),
    (p[2], q[2]), (p[2], q[3]), (p[2], me), (p[3], q[3]), (p[3], me),
    (p[3], q[2]), (me, q[1]), (me, q[2]), (me, q[3]),
]

fn after3(
    said: Int,
    p: List<?Site>,
    q: List<?Site>,
    prev: ?Site,
) -> (List<?Site>, List<?Site>) = match said:
    0 => ([p[0], p[1]], [q[0], q[1], q[2]])
    1 => ([p[1], p[0]], [prev, q[3], q[2]])
    2 => ([q[0], q[1]], [p[2], p[3], prev])
    3 => ([p[2], p[3]], [prev, q[3], q[2]])
    _ => ([prev, p[3]], [q[1], q[2], q[3]])

fn last2(p: List<?Site>, q: List<?Site>, me: ?Site) -> List<(?Site, ?Site)> = [
    (p[0], q[0]), (p[1], q[0]), (p[0], q[1]), (p[1], q[1]), (p[0], q[2]),
    (me, q[2]), (me, q[0]), (me, q[1]), (p[0], me), (p[1], me),
]

fn after2(
    said: Int,
    p: List<?Site>,
    q: List<?Site>,
    prev: ?Site,
) -> (?Site, ?Site, ?Site) = match said:
    0 => (p[1], p[0], q[0])
    1 => (p[1], p[0], q[1])
    2 => (prev, p[0], q[2])
    3 => (q[1], q[0], prev)
    _ => (p[1], p[0], prev)

fn last1(abc: (?Site, ?Site, ?Site), me: ?Site) -> List<(?Site, ?Site)>:
    let (a, b, c) = abc
    [(a, c), (b, c), (a, me), (b, me), (c, me)]

fn spoken(options: List<(?Site, ?Site)>, ends: (?Site, ?Site)) -> Int =
    options.keys.find(k => options[k] == ends or options[k] == (ends.1, ends.0))!

// The first run's state: the tree so far, a diameter's ends u, v (a new site replaces an end
// only when strictly farther, so the ends are always old candidates or sites that joined both
// lists), the candidate lists, the phase plan and the signal it owes, and in the endgame the
// lists the symbols spoken so far leave.
struct Sender:
    n: Int
    plan: List<Phase>
    var tree: JumpTree<Site> = JumpTree<Site>()
    var next_phase = 0
    var p: List<Site> = [Site(0)]
    var q: List<Site> = [Site(0)]
    var pending: ?(Int, Int) = None
    var u: Site = Site(0)
    var v: Site = Site(0)
    var len = 0
    var after_third: (List<?Site>, List<?Site>) = ([], [])
    var after_second: (?Site, ?Site, ?Site) = (None, None, None)

    fn arrive(mut self, at: Site, parent: Site) -> 0..=4:
        tree.add(at, parent)
        if tree.dist(u, at)! > len: (v, len) = (at, len + 1)
        elif tree.dist(v, at)! > len: (u, len) = (at, len + 1)
        if at.ord < n - 3: p.push(at); q.push(at)
        if next_phase < plan.len and plan[next_phase].start == at.ord:
            let phase = plan[next_phase]
            next_phase += 1
            if not (p.contains(u) and q.contains(v)): (u, v) = (v, u)
            var (x, y) = (p.find(u)! / phase.block, q.find(v)! / phase.block)
            if phase.first and x > y: (x, y, u, v) = (y, x, v, u)
            (p, q) = (block(p, x, phase.block), block(q, y, phase.block))
            pending = signal(phase.pairs.find((x, y))!, from: at.ord)
        if let (when, symbol) = pending and when == at.ord: return symbol
        let me: ?Site = Some(at)
        let ends: (?Site, ?Site) = (Some(u), Some(v))
        match n - at.ord:
            3 =>
                let said = spoken(last3(padded(p), padded(q), me), ends) / 5
                after_third = after3(said, padded(p), padded(q), prev: me)
                said
            2 =>
                let said = spoken(last2(after_third.0, after_third.1, me), ends) / 2
                after_second = after2(said, after_third.0, after_third.1, prev: me)
                said
            1 => spoken(last1(after_second, me), ends)
            _ => 0

var sender = Sender(0, [])  // the one value the first run carries from call to call

@grader(two_phase)
fn send_message(n: Int, at: 1..10_000, parent: 0..10_000) -> 0..=4:
    if at == 1: sender = Sender(n, phase_plan(n))
    sender.arrive(Site(at), Site(parent))

@grader(two_phase)
fn longest_path(s: List<0..=4>) -> (Int, Int):
    let n = s.len
    var (p, q): (List<Site>, List<Site>) = ([], [])
    var next = 0
    for phase in phase_plan(n):
        p.extend(Site(next)..=Site(phase.start)); q.extend(Site(next)..=Site(phase.start))
        let (x, y) = phase.pairs[heard(s, phase.start..phase.start + phase.k)]
        (p, q) = (block(p, x, phase.block), block(q, y, phase.block))
        next = phase.start + 1
    p.extend(Site(next)..Site(n - 3)); q.extend(Site(next)..Site(n - 3))
    let (p2, q3) = after3(s[n - 3], padded(p), padded(q), prev: Some(Site(n - 3)))
    let (a, b) = last1(
        after2(s[n - 2], p2, q3, prev: Some(Site(n - 2))),
        Some(Site(n - 1)),
    )[s[n - 1]]
    (a!.ord, b!.ord)
