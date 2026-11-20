// IOI 2012 Day 1 P2, Parachute Rings
// original: corpus/submissions/4689474.cpp

// The graph without one ring: is it still nothing but chains?
struct Without:
    ring: Ring
    parts: Dsu<Ring>
    links: [Ring]Int = 0
    chains: Bool = true
    fn link(mut self, u: Ring, v: Ring):
        if u == self.ring or v == self.ring: return
        self.links[u] += 1
        self.links[v] += 1
        if self.links[u] > 2 or self.links[v] > 2 or not self.parts.unite(u, v): self.chains = false

@grader
struct Rings:
    domain Ring                      // FRICTION: fixed by Init, used by Without above
    adj: [Ring]List<Ring> = []
    parts: Dsu<Ring>
    links: List<(Ring, Ring)> = []
    // Once some ring has three links, only it and its three neighbours can be critical.
    cands: List<Without> = []
    cycles: Int = 0
    cycle_len: Int = 0

    fn Init(n: Int) -> Rings:
        domain Ring = 0..n
        Rings(parts: Dsu<Ring>())

    fn Link(mut self, u: Ring, v: Ring):
        self.links.push((u, v))
        if not self.cands.is_empty:
            // FRICTION: no `for c in mut list`; elements are reached by position to mutate them
            for i in 0..self.cands.len: self.cands[i].link(u, v)
            return
        self.adj[u].push(v)
        self.adj[v].push(u)
        if let hub = first(x for x in [u, v] if self.adj[x].len == 3):
            for ring in [hub] + self.adj[hub]:
                var c = Without(ring, Dsu<Ring>())
                for (a, b) in self.links: c.link(a, b)
                self.cands.push(c)
        elif not self.parts.unite(u, v):
            // all degrees are at most 2, so this component just became a cycle
            self.cycles += 1
            self.cycle_len = self.parts.size(u)

    fn CountCritical(self) -> Int:
        if not self.cands.is_empty: count(c.chains for c in self.cands)
        elif self.cycles == 0: Ring.K
        elif self.cycles == 1: self.cycle_len
        else: 0
