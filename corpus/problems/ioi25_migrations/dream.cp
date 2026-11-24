// Full score needs symbols ≤ 4 and at most 8 messages (N = 10 000). Keep two
// candidate lists for the diameter's ends, P and Q, which start as all sites.
// Five phases shrink them. At a phase's first site, split P and Q into blocks:
// the current ends lie in one block of P and one of Q, and that block pair is
// sent as one signal over the phase's next k sites (silence is pair 0; otherwise
// which site speaks and its symbol 1..=4 spell the index). Sites that arrive
// during the phase may still become ends, so they join both lists. After five
// phases each list has at most 4 sites, and each of the last three sites sends a
// symbol that narrows the end pairs still possible: the editorial's case tables,
// written once and shared by both sides.
use std.math.{iroot}

domain Site = 0..10_000

struct Phase(start: Int, m: Int, b: Int, k: Int, first: Bool):
    @get fn block(self) -> Int = self.m.ceil_div(self.b)
    @get fn pairs(self) -> List<(Int, Int)> =
        [(x, y) for x in 0..self.b for y in (if self.first: x else: 0)..self.b]

fn phase_plan(n: Int) -> List<Phase>:
    for start in 4..n - 3:
        var plan: List<Phase> = []
        var (m, at) = (start - 1, start - 1)
        while at < n - 3 and plan.len < 5:
            let first = plan.is_empty
            let pairs = (b: Int) => if first: b * (b + 1) / 2 else: b * b
            let b = (1..=iroot(8 * (m + 1), 3) + 2).argmin(b => (m + 1).ceil_div(b) + (pairs(b) - 1).ceil_div(4))!
            let k = (pairs(b) - 1).ceil_div(4)
            plan.push(Phase(at, m, b, k, first))
            (m, at) = (m.ceil_div(b) + k - 1, at + k)
        if at == n - 3 and plan.len == 5 and m <= 4: return plan
    assert false, "no start fits five phases"

fn signal(c: Int, from: Int) -> ?(Int, Int) =
    if c > 0: Some((from + (c - 1) / 4, (c - 1) % 4 + 1)) else: None
fn heard(s: List<Int>, window: Range<Int>) -> Int =
    window.first(j => s[j] != 0).map(j => 4 * (j - window.start) + s[j]) ?? 0

fn block(xs: List<Int>, x: Int, size: Int) -> List<Int> = xs.drop(x * size).take(size).collect()

fn last3(p: List<Int>, q: List<Int>, me: Int) -> List<(Int, Int)> = [
    (p[0], q[0]), (p[0], q[1]), (p[0], q[2]), (p[1], q[0]), (p[1], q[1]),
    (p[0], q[3]), (p[0], me),   (p[1], q[2]), (p[1], q[3]), (p[1], me),
    (p[2], q[0]), (p[2], q[1]), (p[3], q[0]), (p[3], q[1]), (me, q[0]),
    (p[2], q[2]), (p[2], q[3]), (p[2], me),   (p[3], q[3]), (p[3], me),
    (p[3], q[2]), (me, q[1]),   (me, q[2]),   (me, q[3])]

fn after3(said: Int, p: List<Int>, q: List<Int>, prev: Int) -> (List<Int>, List<Int>) = match said:
    0 => ([p[0], p[1]], [q[0], q[1], q[2]])
    1 => ([p[1], p[0]], [prev, q[3], q[2]])
    2 => ([q[0], q[1]], [p[2], p[3], prev])
    3 => ([p[2], p[3]], [prev, q[3], q[2]])
    _ => ([prev, p[3]], [q[1], q[2], q[3]])

fn last2(p: List<Int>, q: List<Int>, me: Int) -> List<(Int, Int)> = [
    (p[0], q[0]), (p[1], q[0]), (p[0], q[1]), (p[1], q[1]), (p[0], q[2]),
    (me, q[2]),   (me, q[0]),   (me, q[1]),   (p[0], me),   (p[1], me)]

fn after2(said: Int, p: List<Int>, q: List<Int>, prev: Int) -> (Int, Int, Int) = match said:
    0 => (p[1], p[0], q[0])
    1 => (p[1], p[0], q[1])
    2 => (prev, p[0], q[2])
    3 => (q[1], q[0], prev)
    _ => (p[1], p[0], prev)

fn last1(abc: (Int, Int, Int), me: Int) -> List<(Int, Int)>:
    let (a, b, c) = abc
    [(a, c), (b, c), (a, me), (b, me), (c, me)]

fn same(a: (Int, Int), b: (Int, Int)) -> Bool = a == b or a == (b.1, b.0)

// The tree so far and a diameter's ends u, v: a new site replaces an end only when strictly
// farther, so the ends are always old candidates or sites that joined both lists.
var tree = JumpTree<Site>()
var plan: List<Phase> = []
var phases_done = 0
var (p, q): (List<Int>, List<Int>) = ([], [])
var pending: ?(Int, Int) = None
var (u, v, len) = (0, 0, 0)
var said = 0
var (p2, q3): (List<Int>, List<Int>) = ([], [])

fn say(options: List<(Int, Int)>, per: Int) -> Int:
    said = options.keys.first(k => same(options[k], (u, v)))! / per
    said

@grader(two_phase)
fn send_message(n: Int, at: 1..10_000, parent: 0..10_000) -> 0..=4:
    if at == 1:
        (tree, plan, phases_done, p, q, pending) = (JumpTree<Site>(), phase_plan(n), 0, [0], [0], None)
        (u, v, len) = (0, 0, 0)
    tree.add(Site(at), Site(parent))
    if tree.dist(Site(u), Site(at))! > len: (v, len) = (at, len + 1)
    elif tree.dist(Site(v), Site(at))! > len: (u, len) = (at, len + 1)
    if at < n - 3: p.push(at); q.push(at)
    if phases_done < plan.len and plan[phases_done].start == at:
        let phase = plan[phases_done]
        phases_done += 1
        if not (p.contains(u) and q.contains(v)): (u, v) = (v, u)
        var (x, y) = (p.find(u)! / phase.block, q.find(v)! / phase.block)
        if phase.first and x > y: (x, y, u, v) = (y, x, v, u)
        (p, q) = (block(p, x, phase.block), block(q, y, phase.block))
        pending = signal(phase.pairs.find((x, y))!, from: at)
    if let (when, symbol) = pending and when == at: return symbol
    if n - at > 3: return 0
    match n - at:
        3 => say(last3(p.resized(4, fill: 0), q.resized(4, fill: 0), at), per: 5)
        2 =>
            (p2, q3) = after3(said, p.resized(4, fill: 0), q.resized(4, fill: 0), prev: at - 1)
            say(last2(p2, q3, at), per: 2)
        _ => say(last1(after2(said, p2, q3, prev: at - 1), at), per: 1)

@grader(two_phase)
fn longest_path(s: List<0..=4>) -> (Int, Int):
    let n = s.len
    let s = [Int(x) for x in s]
    var (p, q): (List<Int>, List<Int>) = ([], [])
    var next = 0
    for phase in phase_plan(n):
        p.extend(next..=phase.start); q.extend(next..=phase.start)
        let (x, y) = phase.pairs[heard(s, phase.start..phase.start + phase.k)]
        (p, q) = (block(p, x, phase.block), block(q, y, phase.block))
        next = phase.start + 1
    p.extend(next..n - 3); q.extend(next..n - 3)
    let (p2, q3) = after3(s[n - 3], p.resized(4, fill: 0), q.resized(4, fill: 0), prev: n - 3)
    last1(after2(s[n - 2], p2, q3, prev: n - 2), n - 1)[s[n - 1]]
