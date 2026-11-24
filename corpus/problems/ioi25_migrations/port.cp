// Source: IOI 2025 Day 2 (Migrations), https://github.com/ioi/task-archive/tree/master/2025/day2/migrations
// Problem: N = 10 000 sites join a tree one at a time, site i under P[i] < i. On arrival site i
//   sends a message S[i] in 0..=20000 knowing only the tree so far; afterwards a second run,
//   seeing only S, names two sites at the tree's greatest distance. Full score needs at most 8
//   nonzero messages, all at most 4. Input and output as the IOI sample grader gives them: N
//   and P[1..N-1]; S[1..N-1], then U V.
// Solution: two candidate lists for a diameter's ends start as every site. Five phases split
//   both into blocks; the block pair holding the current ends is spelled over the next k sites
//   (which one speaks and its symbol 1..=4), and sites arriving meanwhile join both lists. The
//   ends are tracked incrementally on a JumpTree: a new site replaces an end only when strictly
//   farther. At most 4 candidates per list remain; the last three sites send symbols from case
//   tables shared by both runs. O(N log N).
// original: corpus/problems/ioi25_migrations/orig.cpp (the official model solution plus the
//   sample grader)
// The judge form is dream.cp's `@grader(two_phase)` fns, linked with dream.tests/grader.cpp;
// the port reads the sample grader's stdin and runs both phases itself, the second on S alone.
// Bounds: n: 10_000..=10_000; parents: [n - 1]0..n
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
            let b = (1..=iroot(8 * (m + 1), 3) + 2).argmin(b => (m + 1).ceil_div(b) + (pairs(b) - 1).ceil_div(4))!
            let k = (pairs(b) - 1).ceil_div(4)
            phases.push(Phase(at, m, b, k, first))
            (m, at) = (m.ceil_div(b) + k - 1, at + k)
        if at == n - 3 and phases.len == 5 and m <= 4: return phases
    assert false, "no start fits five phases"

fn signal(c: Int, from: Int) -> ?(Int, Int) =
    if c > 0: Some((from + (c - 1) / 4, (c - 1) % 4 + 1)) else: None
fn heard(s: List<0..=4>, window: Range<Int>) -> Int =
    window.find(j => s[j] != 0).map(j => 4 * (j - window.start) + s[j]) ?? 0

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
    said = options.keys.find(k => same(options[k], (u, v)))! / per
    said

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

fn longest_path(s: List<0..=4>) -> (Int, Int):
    let sites = s.len
    var (ps, qs): (List<Int>, List<Int>) = ([], [])
    var next = 0
    for phase in phase_plan(sites):
        ps.extend(next..=phase.start); qs.extend(next..=phase.start)
        let (x, y) = phase.pairs[heard(s, phase.start..phase.start + phase.k)]
        (ps, qs) = (block(ps, x, phase.block), block(qs, y, phase.block))
        next = phase.start + 1
    ps.extend(next..sites - 3); qs.extend(next..sites - 3)
    let (ps2, qs3) = after3(s[sites - 3], ps.resized(4, fill: 0), qs.resized(4, fill: 0), prev: sites - 3)
    last1(after2(s[sites - 2], ps2, qs3, prev: sites - 2), sites - 1)[s[sites - 1]]

let n: 10_000..=10_000 = read()
let parents: [n - 1]0..n = read()
for i in 1..n: assert parents[i - 1] < i, "P[i] must be below i"
var s: List<0..=4> = [0]
for i in 1..n: s.push(send_message(n, i, parents[i - 1]))
print(s.drop(1))
let (end1, end2) = longest_path(s)
print(end1, end2)
