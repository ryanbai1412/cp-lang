// Source: CCO 2022 Day 2 (Phone Plans), https://dmoj.ca/problem/cco22p5
// limits: 3000 ms, 1024 MB
// Problem: N <= 2e5 houses; companies A and B each have up to 2e5 lines (u, v, level <= 1e9).
//   Buying level l from a company gives its lines of level <= l, for price l (0 allowed). Two
//   houses can talk iff joined by one company's lines. Print the least total price that makes
//   at least K pairs talk, or -1.
// Solution: two pointers: raise A's level line by line while lowering B's. B is built with all
//   its lines in a rollback DSU, so lines come off from the top. Talking pairs = pairs joined in
//   A + pairs joined in B - pairs joined in both, the last counted by a HashMap over each house's
//   (A group, B group), updated for the houses a merge or an undo moves (union by size, so each
//   house moves O(log N) times). After each A line drop B lines while the count stays >= K; the
//   last B line dropped prices B. O(N log^2 N + A + B) expected.
use std.dsu.{Mark}

let (n, ma, mb): (1..=2e5, 0..=2e5, 0..=2e5) = read()
let k: 0..=2e10 = read()
domain House = 1..=n
domain LineA = 0..ma
domain LineB = 0..mb
let a = read<[LineA](u: House, v: House, w: 0..=1e9)>().sorted(by: l => l.w)
let b = read<[LineB](u: House, v: House, w: 0..=1e9)>().sorted(by: l => l.w)

// One company's bought lines: union by size, so a merge moves the smaller group's houses.
struct Net:
    var dsu: Dsu<House>
    var members: [House]List<House>
    var pairs: Int

    // Buy line u-v; the houses whose group changed.
    fn link(mut self, u: House, v: House) -> List<House>:
        if let (kept, merged) = dsu.merge(u, v):
            let moved = members[merged].clone()
            members[merged].clear()
            members[kept] += moved
            pairs += moved.len * (members[kept].len - moved.len)
            return moved
        []

fn alone(rollback: Bool) -> Net = Net(Dsu<House>.new(rollback), [House] h => [h], 0)
var net_a = alone(rollback: false)
var net_b = alone(rollback: true)
var history: List<(mark: Mark, pairs: Int, moved: List<House>)> = []
for l in b:
    let (mark, pairs) = (net_b.dsu.mark(), net_b.pairs)
    history.push((mark, pairs, net_b.link(l.u, l.v)))

var group = [House] h => (net_a.dsu.find(h), net_b.dsu.find(h))
var together = HashMap<(House, House), Int>()
var both = 0
fn enter(h: House):
    let c = together.get(group[h]) ?? 0
    both += c
    together[group[h]] = c + 1
fn regroup(h: House):
    together[group[h]] -= 1
    both -= together[group[h]]
    group[h] = (net_a.dsu.find(h), net_b.dsu.find(h))
    enter(h)
for h in House: enter(h)
fn talking() -> Int = net_a.pairs + net_b.pairs - both

var bought_b = mb
var best: ?Int = None
fn relax(price_a: Int):
    while bought_b > 0 and talking() >= k:
        bought_b -= 1
        let step = history[bought_b]
        net_b.dsu.undo_to(step.mark)
        net_b.pairs = step.pairs
        for h in step.moved: regroup(h)
    if talking() >= k: best min= price_a
    if bought_b < mb: best min= price_a + b[bought_b].w

relax(0)
for l in a:
    for h in net_a.link(l.u, l.v): regroup(h)
    relax(l.w)
print(best, none: -1)
