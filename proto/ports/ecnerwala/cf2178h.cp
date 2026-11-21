// Codeforces 2178 H: Create or Duplicate.
// Build every type's count by Horner's rule: its first create is the free
// initial present, and a duplicate doubles every type already started, so
// the state is the value sum mod m and the set of started types. Each move
// has a fixed cost, so Dijkstra needs no heap: per move, the settled states
// in order give its candidates in order of distance.
domain T = 0..3
enum Move: Create(T), Double(1..=3)

@!multitest
let x: [T]1..=500_000 = read()
let (m, k): (2..=500_000, 1..=500_000) = read()
domain R = 0..m
type S = Subset<T>
fn cost(mv: Move) -> Int = match mv:
    Create(t) => x[t]
    Double(j) => j * k
fn step(mv: Move, r: R, s: S) -> ?(R, S) = match mv:
    Create(t) => (R((r.to_idx + x[t]) % m), s.with(t))
    Double(j) => if s.len == j: (R(2 * r.to_idx % m), s) else: none

// Monotone-queue Dijkstra for fixed-cost move classes; each cursor `at[i]` stays
// on its first untaken candidate in `order` and advances once that candidate is taken.
fn fixed_cost_paths(moves: List<Move>) -> Int:
    var dist: [R, S]?Int = none
    dist[R(0), S.empty] = 0
    var order = [(R(0), S.empty)]
    var at: [0..moves.len]Int = 0
    while dist[R(0), S.full] == none:
        var best: ?(d: Int, i: Int, r: R, s: S) = none
        for i in 0..moves.len:
            var head = at[i]
            while head < order.len:
                let (r, s) = order[head]
                if let (r2, s2) = step(moves[i], r, s) and dist[r2, s2] == none:
                    best min= (dist[r, s]! + cost(moves[i]), i, r2, s2)
                    break
                head += 1
            at[i] = head
        let (d, i, r, s) = best!
        dist[r, s] = d
        order.push((r, s))
        at[i] += 1
    dist[R(0), S.full]!

let moves = [Create(t) for t in T] + [Double(j) for j in 1..=3]
print(fixed_cost_paths(moves) - x.sum())
