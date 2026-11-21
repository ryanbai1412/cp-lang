// Codeforces 2137 G: Cry Me a River.
// Two players alternate moving a token along a DAG; Cry (first) wins by
// reaching a sink without touching a red vertex, River (second) wins by
// forcing the token onto a red vertex.
// Each vertex has one of four outcomes `(cry_as_first, cry_as_second)`:
//   - `BothCry`    (T, T): Cry wins regardless of turn (e.g. non-red sink).
//   - `FirstWins`  (T, F): mover wins (has a `SecondWins` out-neighbor, or both
//                          a `BothCry` and a `BothRiver` out-neighbor).
//   - `SecondWins` (F, T): mover loses (every out-neighbor is `FirstWins`).
//   - `BothRiver`  (F, F): River wins regardless of turn (red vertex, or only
//                          `FirstWins`/`BothRiver` neighbors with >= 1 `BothRiver`).
// Painting vertices red can only flip each player's win bit `true -> false`, so
// every vertex changes outcome at most twice, bounding `propagate` to O(n + m).

enum Outcome: BothCry | FirstWins | SecondWins | BothRiver

fn code(o: Outcome) -> 0..4 = match o:
    Outcome.BothCry => 0
    Outcome.FirstWins => 1
    Outcome.SecondWins => 2
    Outcome.BothRiver => 3

@!multitest
let (n, m, q): (1..=200_000, 0..=200_000, 1..=200_000) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let g = Graph.directed(edges)
let rev = g.transpose()

var state: [V]Outcome = Outcome.BothCry
var freq: [V, 0..4]Int = 0
var is_red: [V]Bool = false

fn eval(v: V) -> Outcome:
    if is_red[v]: return Outcome.BothRiver
    if g.deg(v) == 0: return Outcome.BothCry
    let has_c = freq[v, code(Outcome.BothCry)] > 0
    let has_r = freq[v, code(Outcome.BothRiver)] > 0
    let has_s = freq[v, code(Outcome.SecondWins)] > 0
    if has_s or (has_c and has_r): return Outcome.FirstWins
    if has_r: return Outcome.BothRiver
    if has_c: return Outcome.BothCry
    Outcome.SecondWins

for v in peel(rev):
    state[v] = eval(v)
    for a in rev.out(v): freq[a.to, code(state[v])] += 1

fn propagate(v: V):
    let old_s = state[v]
    let new_s = eval(v)
    if new_s == old_s: return
    state[v] = new_s
    for a in rev.out(v):
        freq[a.to, code(old_s)] -= 1
        freq[a.to, code(new_s)] += 1
        propagate(a.to)

repeat q:
    let (op, v): (1..=2, V) = read()
    if op == 1:
        is_red[v] = true
        propagate(v)
    else:
        let cry_wins = state[v] == Outcome.BothCry or state[v] == Outcome.FirstWins
        print(if cry_wins: "YES" else: "NO")
