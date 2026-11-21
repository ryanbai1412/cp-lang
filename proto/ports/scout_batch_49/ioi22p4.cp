// IOI 2022 Day 2 P1, Digital Circuit
// original: corpus/submissions/4783516.cpp
const MOD = 1_000_002_022
type Z = Mod<MOD>
type Lit = (on: Z, all: Z)           // worth of the lit sources, worth of all sources

@grader
struct Circuit:
    domain Node, Source              // FRICTION: Source is the tail n.. of Node, tied together by hand below
    gates: Int
    lit: LazySegTree<Source, Lit>

    fn init(n: Int, m: Int, parent: [Node]Int, state: [Source]0..=1) -> Circuit:
        domain Node = 0..n + m
        domain Source = 0..m
        var kids: [Node]List<Node> = []
        for v in Node if let u = Node.from_idx(parent[v]): kids[u].push(v)

        // all[u]: parameter assignments inside u's subtree (a gate with c inputs has c choices)
        var all: [Node]Z = 1
        for u in Node.rev if not kids[u].is_empty:           // parents are numbered before children
            all[u] = kids[u].len
            for v in kids[u]: all[u] *= all[v]               // FRICTION: there is `it.sum()` but no `product(it)`

        // A lit source is worth the assignments of everything off its path to
        // the root: by linearity each gate on the path passes the count on once.
        // MOD is not prime, so "all but one child" cannot divide: it is a
        // prefix product times a suffix product.
        var worth: [Node]Z = 1
        for u in Node:
            let c = kids[u].len
            var after: [0..=c]Z = 1
            for i in (0..c).rev: after[i] = after[i + 1] * all[kids[u][i]]
            var before = worth[u]
            for (i, v) in kids[u].items:
                worth[v] = before * after[i + 1]
                before *= all[v]

        let leaves = [Source] s =>
            let w = worth[Node.from_idx(n + s.to_idx)!]
            (on: w * state[s], all: w)
        let flip = Action(false, (f, g) => f != g, (f, x) => if f: (x.all - x.on, x.all) else: x)
        Circuit(n, LazySegTree<Source, Lit>((0, 0), (a, b) => (a.on + b.on, a.all + b.all), flip, leaves))

    fn count_ways(mut self, l: Int, r: Int) -> Z:
        self.lit.apply(Source.from_idx(l - self.gates)!..=Source.from_idx(r - self.gates)!, true)
        self.lit.fold(..).on
