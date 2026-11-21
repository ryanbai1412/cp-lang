// Codeforces 2178 H: Create or Duplicate (dream).
// Build every type's count by Horner's rule: its first create is the free
// initial present, and a duplicate doubles every type already started, so
// the state is the value sum mod m and the set of started types. Every move
// has a fixed cost, so the shortest path needs no heap.
domain T = 0..3
enum Move: Create(T), Double(1..=3)

--- multitest ---
let x: [T]1..=5e5 = read()
let (m, k): (2..=5e5, 1..=5e5) = read()
domain R = Zmod(m)
domain State = (r: R, s: Subset<T>)

fn cost(mv: Move) -> Int = match mv:
    Create(t) => x[t]
    Double(j) => j * k
fn step(mv: Move, (r, s): State) -> ?State = match mv:
    Create(t) => (r + x[t], s.with(t))
    Double(j) => (2 * r, s) if s.len == j

let dist: [State]?(0..=2e9) = fixed_cost_paths(from: (R(0), {}), moves: Move.all, cost, step)
print(dist[(R(0), T.all)]! - sum(x))
