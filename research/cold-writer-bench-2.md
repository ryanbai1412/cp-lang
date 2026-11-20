# Cold-writer benchmark #2

Base: `main` at `c120539`, after Syntax Batches #1–#4. Ten standard contest
problems, each written once in the most natural 0.2.2 style I could manage
without looking anything up. The **exact first drafts** were then run through
`cpl check`, `cpl build`, `g++ -std=c++17 -O2`, and the sample through both
`cpl run` and the compiled binary. A draft passes end to end only when all
five steps succeed and both outputs match the expected answer.

No earlier first-try figure is recorded in the repo or in the factory metrics
log, so this run is the baseline for metric #3.

## Result

- **First-try `cpl check` pass rate: 7/10.**
- **First-try end-to-end pass rate: 7/10.** Every draft that checked also
  built, compiled with g++, and printed the right answer from both `cpl run`
  and the binary. There were no wrong answers and no run/build differences.
- **After one small edit: 9/10.** Two of the three failures were one-word fixes
  (an import and a capital letter). The grid BFS needed a rewrite.

The ten problems: frog DP (AtCoder DP A), interval DP (DP N "Slimes"), tree
rerooting (CSES "Tree Distances II"), grid BFS, max flow (CSES "Download
Speed"), segment tree (CSES "Dynamic Range Minimum Queries"), binomials mod P
(CSES "Binomial Coefficients"), string matching (CSES "String Matching"),
greedy by end time (CSES "Movie Festival") and a constructive one (CSES
"Permutations").

## Passed first try (7)

These read the way a contestant would write them, with no fights at all. Some
lines worth keeping as the reference idiom:

```cp
// frog DP: a domain step is optional, and min= on ?Int starts from none
for i in I:
    for k in 1..=2:
        if let j = i + k:
            dp[j] min= dp[i]! + abs(h[j] - h[i])

// rerooting: Rooted straight from an undirected Graph, print a [V]Int on one line
let t = Rooted(Graph<V, E>.undirected(edges), V.first)
for v in t.postorder:
    if let p = t.parent(v): size[p] += size[v]
ans[V.first] = sum(t.depth(v) for v in V)
for v in t.preorder:
    if let p = t.parent(v): ans[v] = ans[p] + n - 2 * size[v]
print(ans)

// max flow, binomials, greedy
var f = FlowGraph<V>.new()
for p in pipes: f.add(p.a, p.b, p.c)
print(f.max_flow(V.first, V.last))

let comb = Comb<P>(1000000)
print(comb.choose(a, b))

for m in sorted(movies, by: m => m.b):
    if m.a >= end: ...
```

Interval DP (`prefix(a).sum(I(l)..=I(r))`, `best min= ...`) and the
constructive permutation (list comprehensions joined with `+`, printed as one
line) also passed unchanged.

## The three first-draft fights

### 1. Grid BFS: `bfs` cannot search `(R, C)` vertices (real gap, the worst one)

The natural draft:

```cp
fn nbrs(p: (R, C)) -> yields (R, C):
    let (r, c) = p
    for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
        if let nr = r + dr and let nc = c + dc:
            if g[nr][nc] != '#': yield (nr, nc)
let dist = bfs(nbrs, [s])
print(dist[e] ?? -1)
```

fails with `error[E0201]: cpl run doesn't know the name V here` at **line 1,
column 1**. The error comes from inside the generic `bfs`, names a type
parameter the user never wrote, and points at the top of the file. Rewriting
`nbrs` with `Dir4.all()` and `p.step(d)` fails the same way. Wave 5 found
the same gap for `(F, Bool)` states, and wave 7 found four more ports that
pack 2D states into one domain to get around it.

Getting the grid BFS to work took three more fights:

- `Deque.of(s)` needs a list: `Deque.of([s])`.
- There is no `push_back`; the method is `push`. The error gives no hint.
- `p.step(d)` needs `use std.grid.{step}`, even though `Dir4` from the same
  module is in the prelude.

The version that works is an 11-line hand-written BFS loop where 2 lines
should do:

```cp
use std.grid.{step}
var dist: [R, C]?Int = none
var todo = Deque.of([s])
dist[s] = 0
while let p = todo.pop_front():
    for d in Dir4.all():
        if let q = p.step(d):
            if g[q.0][q.1] != '#' and dist[q] == none:
                dist[q] = dist[p]! + 1
                todo.push(q)
```

### 2. String matching: `find_all` is not in the prelude (stale prelude)

`print(find_all(s, p).len)` fails with `no find_all in scope`. Adding
`use std.strings.{find_all}` fixes it, and then everything else just works:
a `Str` is accepted as `List<T>`, and `.len` on a generator counts it. The
prelude imports `repeat`, `RollingHash` and `HashVal` from `std.strings`
but not `find_all`, `z_function` or `prefix_function`, which are the ones a
contestant reaches for first.

### 3. Segment tree: `Min` versus `min` (naming inconsistency)

`SegTree<I, Int>(Min, x)` fails with `no Min in scope`. The answer is the
bare function, `SegTree<I, Int>(min, x)`. The capital came from the lazy
tree, whose stock actions are capitalised (`AddMin`, `AssignMax`), so the two
trees teach opposite habits. The error does not suggest `min`.

## Proposed fixes, in order

1. **Graph search over tuple vertices.** `bfs`, `dijkstra`, `topo` and
   `peel` accept `(A, B)` vertices when `A` and `B` are finite domains, with
   the result indexed as `[A, B]?Int`. This is the only failure that needed a
   rewrite. It recurs in waves 5 and 7 and is the main reason ports pack states
   into one domain by hand.
2. **Prelude gate for module free functions.** Export `find_all`,
   `z_function`, `prefix_function` and `grid.step`. Add a test that the
   name-not-found error suggests `use std.<module>.{name}` whenever a public
   stdlib item has that name.
3. **"Did you mean" hints.** Case-only mismatches (`Min` → `min`) and
   common names from other languages (`push_back` → `push`,
   `Deque.of(x)` → `Deque.of([x])`).
4. **Point generic-instantiation errors at the user's call**, not at 1:1. This
   is already in the master plan from wave 4; it came up again here.

## What was not covered

There was no interactive problem this round; the tenth slot was constructive.
Only the samples were run, with no random stress, so this measures how easily
the code is written, not how correct it is under stress.
