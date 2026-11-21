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

@grader(two_phase)
mod migrations:
    type Site = 0..10_000

    struct Phase(start: Site, m: Int, b: Int, k: Int, first: Bool):
        fn block(self) -> Int = ceil_div(self.m, self.b)
        // Block pairs in signal order; in the first phase P = Q, so pairs are unordered.
        fn pairs(self) -> List<(Int, Int)> =
            [(x, y) for x in 0..self.b for y in (if self.first: x else: 0)..self.b]

    fn phase_plan(n: Int) -> List<Phase>:
        for start in 4..n - 3:
            var plan: List<Phase> = []
            var (m, at) = (start - 1, start - 1)
            while at < n - 3 and plan.len < 5:
                let first = plan.is_empty
                fn pairs(b: Int) -> Int = if first: b * (b + 1) / 2 else: b * b
                let b = (1..=2 * iroot(m + 1, 3) + 3)
                    .min_by(b => ceil_div(m + 1, b) + ceil_div(pairs(b) - 1, 4))
                let k = ceil_div(pairs(b) - 1, 4)
                plan.push(Phase(at, m, b, k, first))
                (m, at) = (ceil_div(m, b) + k - 1, at + k)
            if at == n - 3 and plan.len == 5 and m <= 4: return plan
        unreachable

    // A signal is an index c sent over a window: 0 is silence, otherwise
    // site (c − 1) / 4 of the window says (c − 1) % 4 + 1.
    fn signal(c: Int, from: Site) -> ?(Site, 1..=4) =
        if c > 0: (from + (c - 1) / 4, (c - 1) % 4 + 1) else: none
    fn heard(s: [Site]0..=4, window: Range<Site>) -> Int =
        window.first(where: j => s[j] != 0).map(j => 4 * (j - window.start) + s[j]) ?? 0

    // The endgame. Each table lists the end pairs still possible; a site sends its
    // pair's index divided by the group size, and `after` says what that leaves.
    fn last3(p: [4]Site, q: [4]Site, me: Site) -> List<(Site, Site)> = [   // 5 per symbol
        (p[0], q[0]), (p[0], q[1]), (p[0], q[2]), (p[1], q[0]), (p[1], q[1]),
        (p[0], q[3]), (p[0], me),   (p[1], q[2]), (p[1], q[3]), (p[1], me),
        (p[2], q[0]), (p[2], q[1]), (p[3], q[0]), (p[3], q[1]), (me, q[0]),
        (p[2], q[2]), (p[2], q[3]), (p[2], me),   (p[3], q[3]), (p[3], me),
        (p[3], q[2]), (me, q[1]),   (me, q[2]),   (me, q[3])]

    fn after3(said: 0..=4, p: [4]Site, q: [4]Site, prev: Site) -> ([2]Site, [3]Site) = match said:
        0 => ([p[0], p[1]], [q[0], q[1], q[2]])
        1 => ([p[1], p[0]], [prev, q[3], q[2]])
        2 => ([q[0], q[1]], [p[2], p[3], prev])
        3 => ([p[2], p[3]], [prev, q[3], q[2]])
        4 => ([prev, p[3]], [q[1], q[2], q[3]])

    fn last2(p: [2]Site, q: [3]Site, me: Site) -> List<(Site, Site)> = [   // 2 per symbol
        (p[0], q[0]), (p[1], q[0]), (p[0], q[1]), (p[1], q[1]), (p[0], q[2]),
        (me, q[2]),   (me, q[0]),   (me, q[1]),   (p[0], me),   (p[1], me)]

    fn after2(said: 0..=4, p: [2]Site, q: [3]Site, prev: Site) -> (Site, Site, Site) = match said:
        0 => (p[1], p[0], q[0])
        1 => (p[1], p[0], q[1])
        2 => (prev, p[0], q[2])
        3 => (q[1], q[0], prev)
        4 => (p[1], p[0], prev)

    fn last1((a, b, c): (Site, Site, Site), me: Site) -> List<(Site, Site)> =
        [(a, c), (b, c), (a, me), (b, me), (c, me)]                          // 1 per symbol

    fn same(a: (Site, Site), b: (Site, Site)) -> Bool = a == b or a == (b.1, b.0)

    var tree: Graph<Site>
    var plan: List<Phase>
    var p, q: List<Site>
    var pending: ?(Site, 1..=4)
    var said = 0
    var (p2, q3): ([2]Site, [3]Site)

    fn send_message(n: Int, i: Site, parent: Site) -> 0..=4:
        if i == 1: (tree, plan, p, q, pending) = (Graph(n), phase_plan(n), [0], [0], none)
        tree.add_edge(i, parent)
        if i < n - 3: p.push(i); q.push(i)
        if plan.first?.start == i:
            let phase = plan.pop_front()
            var (u, v) = tree.diameter()                   // O(n) BFS, at 8 sites only
            if u not in p or v not in q: (u, v) = (v, u)
            var (x, y) = (p.index_of(u)! / phase.block, q.index_of(v)! / phase.block)
            if phase.first: (x, y) = minmax(x, y)
            (p, q) = (p.chunk(x, size: phase.block), q.chunk(y, size: phase.block))
            pending = signal(phase.pairs.index_of((x, y))!, from: i)
        if let (at, symbol) = pending and at == i: return symbol
        if n - i > 3: return 0
        let ends = tree.diameter()
        fn say(options: List<(Site, Site)>, per: Int) -> 0..=4 =
            said = options.position(where: c => same(c, ends))! / per
            said
        match n - i:
            3 => say(last3(p.padded(4, with: 0), q.padded(4, with: 0), i), per: 5)
            2 =>
                (p2, q3) = after3(said, p.padded(4, with: 0), q.padded(4, with: 0), prev: i - 1)
                say(last2(p2, q3, i), per: 2)
            1 => say(last1(after2(said, p2, q3, prev: i - 1), i), per: 1)

    fn longest_path(s: [Site]0..=4) -> (Site, Site):
        let n = s.len
        var p, q: List<Site> = []
        var next = 0
        for phase in phase_plan(n):
            p.extend(next..=phase.start); q.extend(next..=phase.start)
            let (x, y) = phase.pairs[heard(s, phase.start..phase.start + phase.k)]
            (p, q) = (p.chunk(x, size: phase.block), q.chunk(y, size: phase.block))
            next = phase.start + 1
        p.extend(next..n - 3); q.extend(next..n - 3)
        let (p2, q3) = after3(s[n - 3], p.padded(4, with: 0), q.padded(4, with: 0), prev: n - 3)
        last1(after2(s[n - 2], p2, q3, prev: n - 2), n - 1)[s[n - 1]]
