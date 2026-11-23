# agc077c

Score: 4/5
Dream: 4/5

The complement-graph components are found by a hand DFS with a two-pointer skip over sorted adjacency, and the parity bookkeeping (`out_deg`, `c0`/`c1`, `mi_1`/`ma_1`) is never explained. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- No invariant header: why even `k` is always yes, what `out_deg` parity means, and why the answer is `n / 2` inside `[mi_1, ma_1]`.
- Raw index space (lines 3-7): `[0..n]` with `u0 - 1`; no vertex domain.
- Unexplained initial parity (line 4): `(n - 1 - i) % 2`, and the `u > v` flip (lines 10-12).
- Hand complement traversal (lines 26-35): `unvisited.first_ge`, a manual adjacency pointer `ai` and recursion over raw `Int`.
- Paired counters `c0`/`c1` and unexplained closing terms (lines 38-39): `c1 % 2`, `tot - (tot - c1) % 2`.

## Owed rewrites

- Header with the parity argument; a `V` domain; name the complement components; derive the per-component range.

## Language gaps

No friction report exists. Possibly missing: complement-graph components in `std.graph`; needs a fresh-compiler check before filing.

## Cold review (HEAD)

Dream 3/5 (new, d93c0286), sent back.
- Line 44 checks only `lo <= target <= hi`, but line 11 says each component reaches only counts *of the same parity as `odd`*. Either the parity of `target` is forced, and the header must say why, or the check is missing it.
- Lines 7-8 state the k-even and k-odd conditions without deriving either.
- `[v for v in comp if odd_out[v]].len` (line 39) should be `comp.count(v => odd_out[v])`, which works today.
- `complement_components` (line 37) isn't in the stdlib (E0201); the friction report covers it.

Port 2/5: it hand-rolls the complement-graph DFS with an unvisited set, a `cursor` and a merge pointer `ai` (the reported gap).

## Cold review (HEAD)

Dream 3 → 4/5, rescored after ce8206db. The header now proves that the parity of the odd-vertex count is fixed (`n(n-1)/2 ≡ floor(n/2)`), so `lo <= target <= hi` is enough; the parity concern is closed.
- Owed for a 5: the even-`k` case (lines 7-8) is still only asserted.
- `std.graph.complement_components` (line 20) isn't in the stdlib (E0201); file it in the friction report if it isn't there.

The port score (2/5) holds; it doesn't use the dream's component intervals yet.

## Cold review (HEAD, port rescore)

Port pending → 3/5, rescored cold on current main (dream stays 4/5). The header now carries the dream's full parity proof, and the paired edge loop is gone.
- Blocked by a reported gap: the port hand-writes `complement_components` (lines 41-60) because `std.graph.complement_components` doesn't exist yet. The hand version is a cursor walk with a second pointer `ai` into a sorted adjacency list (lines 49-57).
- Owed now: lines 36-39 build and sort the adjacency by hand, where the dream uses `undirected_of_pairs`. `count(odd_out[v] for v in comp)` (line 66) is a canonical generator fold (§53.1).

## Cold review (a7c4810b)

Port 3/5 (reported gap), rescored cold on current main; dream 4/5 holds. The owed adjacency item is fixed: `Graph.undirected(edges)` (line 36) replaces the hand-built lists. The port checks with zero errors.
- Gap, reported in the friction file: `complement_components` (lines 39-58) is still hand-written as a cursor walk over `unvisited` with a merge pointer `ai`, so the port stays at 3 until `std.graph` has it.
- Minor: the walk (lines 47-55) can be `while let at = cursor and let nxt = unvisited.first_ge(at):`, which drops the `else: break` (probed).
- Minor: `dfs` recurses once per vertex of a component, up to `n = 200_000` deep; say the judge's stack covers it.
- Minor: the even-`k` case (lines 7-8) is still only asserted.

## Cold rescore (c7798b6c1)

Port 3/5 (reported, measured gap), rescored cold after 3a822d3c5; dream 4/5 holds. `cpl check` and `cpl lint` are clean.

Better:
- The cursor walk, merge pointer `ai` and n-deep recursion are gone: `complement_components` (lines 38-53) is the standard unvisited-list BFS (stamp `near`, `partition`, queue the rest), O(n + m) as the header (lines 15-17) argues.
- Statement bounds are types now (`n: 4..=200_000`, `k: 2..=n - 2`, lines 21-23), and `yes()` under `with io(answers:)` replaces the string literals.
- The header derives the even-`k` parity flip (lines 6-7), and the closing `lo <= target <= hi` (line 64) follows from the parity argument (lines 13-15).

Still fails:
- Hand-rolled standard algorithm (lines 38-53): `components(complement(joined))` checks but is O(n^2) (28.6 s on n = m = 2e5, friction file), so the BFS stays hand-written. Reported and measured, which is the definition of a 3; it reaches 4 once `std.graph` gives `Complement` searches the unvisited-set walk its doc promises.
- Owed, mixed spelling: line 42 binds `while let s = unvisited.pop()`, line 46 `while let Some(u) = queue.pop_front()`; `while let u = queue.pop_front():` checks and runs (probed).
- Owed: order is irrelevant, so the `Deque` (lines 44-46, 50) can be a stack, `var todo = [s]` / `while let u = todo.pop():` / `todo.extend(apart)`; probed, agrees with the port on 150 `gen.py` inputs.
- Owed: the even-`k` exit (lines 32-34) runs after the parity setup (lines 27-30) it discards; move it right after `let edges` (probed, same 150 inputs).
- Minor: the even-`k` "always Yes" (lines 7-8) still cites the editorial instead of deriving it, and line 8 is a stray short wrap.

Dream notes: its bounds (`n: 1..`, `k: 1..=1_000_000_000`, lines 24-26) are looser than the statement the port now encodes, and it prints with `if ...: "Yes" else: "No"` where the port has `yes()`.

## Cold rescore (57f3059f2)

Port 3 → 4/5, rescored cold after 3e1dc1074; dream 4/5 holds. `cpl check` and `cpl lint` are clean, the sample passes.

Better:
- The hand-written complement BFS is gone: line 39 is `components(complement(Graph.undirected(edges)))`, the dream's named algorithm, and the friction file measures it at 0.022-0.033 s against orig.cpp's 0.078-0.095 s on three max inputs.
- The even-`k` exit (lines 30-32) now runs before the parity setup, and the header derives it (lines 10-11): the operations on S + a and S + b flip just a and b.
- The header carries the whole argument (lines 5-18): the odd-count parity identity, the per-component interval and why `lo <= target <= hi` (line 49) suffices. Bounds are types (lines 24-26), output is `yes()`.

Still fails (all owed, minor; none needs the language):
- Parallel per-component counters (lines 40-44): `size` and `odd` are two `[C]Int` tables filled by one loop, where the dream reads each component's members. `for members in comp.buckets():` with `members.len` and `members.count(v => odd_out[v])` checks, lints clean and agrees with the port on 120 `gen.py` inputs (probed).
- One fact, two spellings: the header gives the start parity as `(n - v) % 2` (line 13), line 34 as `(n - 1 - v.ord) % 2`; `(n - Int(v)) % 2 == 1` checks and agrees on 60 inputs (probed).
- Lines 36-37 spell the toggle `x = not x` twice; `odd_out[u] ^= true` checks (probed). Cosmetic.

Dream notes: unchanged from the previous rescore (even `k` only asserted on lines 7-8, bounds looser than the statement, `"Yes"`/`"No"` literals where the port has `yes()`).
