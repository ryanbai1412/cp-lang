# 1.1 Overview and a tour

Status: decided except where 1.2-1.8 mark items as proposed. This is a
quick tour of the whole language through one program. Details are in
docs 1.2-1.8.

## A complete program

Shortest paths on a weighted graph, with several tests per file:

```
use std.graph
use std.heap
@multitest(sum: n <= 2e5)

let (n, m): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 1..=m
let edges: [E](u: V, v: V, w: 1..=1e9) = read()
let g = Graph<V>.undirected(edges)

fn dijkstra(src: V) -> [V]?Int:
    var dist: [V]?Int = none
    var pq = Heap<(Int, V)>(by: min)
    dist[src] = 0
    pq.push((0, src))
    while let (d, u) = pq.pop():
        if d != dist[u]: continue
        for (v, e) in g.adj(u):
            let nd = d + edges[e].w          // proven < 2e14: no overflow check
            if nd < (dist[v] ?? Int.MAX):
                dist[v] = nd
                pq.push((nd, v))
    dist

let dist = dijkstra(V.first)
print(" ".join(dist[v] ?? -1 for v in V))
```

What to notice:

- **The file is a function body.** Top-level code runs once per test under
  `@multitest`. Top-level fns see the file's `let`s (`edges`, `g`), so they
  need no parameters for the input. Removing `@multitest` makes it a
  single-test program, and nothing else changes.
- **Domains.** `V` and `E` are different index spaces. `dist[e]` or
  `edges[v]` doesn't compile, and `dist[v]` needs no bounds check.
- **Typed input.** The ranges give the compiler the bounds it uses for
  storage widths (`w` takes 4 bytes), overflow proofs, cost estimates and
  input validation.
- **Optionals.** `?Int` uses a spare value, so it costs nothing extra.
  `none` replaces the `INF` sentinel.
- **Operation values.** `Heap(by: min)` takes the ordering as a value, and
  the compiler specializes the heap on it.
- **No hidden costs.** Every loop compiles to what you'd write in C++.

## A second program: parts, a brute, and bounds

Count subarrays whose sum is at most `k`, written as a fast part and a
brute, with prefix sums on bounds so the usual `+1` shifts can't go wrong:

```
let (n, k): (1..=2e5, 0..=1e14) = read()
domain I = 1..=n
let a: [I]0..=1e9 = read()

var pre: [I.cuts]Int = 0                // n + 1 boundaries between elements
for i in I: pre[i.r] = pre[i.l] + a[i]

@brute(count_brute)
fn count() -> Int:
    var total = 0
    for (w, _) in sliding(I, init: 0,
            add: (sum, i) => sum + a[i],
            remove: (sum, i) => sum - a[i],
            ok: sum => sum <= k):
        total += w.len                    // valid windows ending at this right end
    total

fn count_brute() -> Int =
    sum(1 for l in I for r in I if l <= r and pre[r.r] - pre[l.l] <= k)

print(count())
```

What to notice:

- **Positions and bounds are different types.** `pre` is indexed by the
  boundaries of `I` (`i.l`, `i.r`), so `pre[i]` doesn't compile
  and the classic `pre[r] - pre[l - 1]` vs `pre[r + 1] - pre[l]` mix-up
  can't happen.
- **Sequence views** like `sliding` give each window as a range (with `len`,
  `first`, `last`), so the pointer movement and the empty window are in
  the stdlib, not in your loop, and still compiles to the hand-written
  loop.
- **Parts and brutes.** `@brute` pairs the fast part with a slow one.
  `cp stress d` generates inputs from the declared types, runs both,
  and shrinks any mismatch to the smallest input, showing it as it
  shrinks. The brute is removed from the submission.
- **Overflow.** The compiler knows `pre` reaches at most `2e14`, and
  `total` at most `n * (n + 1) / 2`, so no overflow check is needed.

## The concepts, one line each

- **Values:** `let`/`var`, `Int` (64-bit), range types, `?T`, records,
  structs, enums (1.2).
- **Domains:** typed index spaces, bounds, `Range`/`Range1`, slices,
  generators and sequence views such as `runs`, `windows` and `pairs` (1.3).
- **Abstraction:** traits for behaviour with one canonical instance,
  operation values for behaviour with many, laws, and generics specialized
  at compile time (1.4).
- **State:** variables own, parameters borrow, `mut` at both ends,
  `.copy()`, pools and handles, `defer`, rollback (1.5).
- **Program structure:** program and library files, `@multitest`,
  `static let`, modules (1.6).
- **I/O:** typed `read()`, `print` with interpolation, interactors (1.7).
- **Syntax:** indentation blocks, `=>` for lambdas, tables and match arms;
  `and`/`or`/`not` (1.8).

## What the tools add

`cp run d i1` runs the program in the interpreter, with no g++ wait, and
checks bounds, ranges and overflow. `cp stress d` compares the program
against a brute force and shrinks any failing input. `cp build` emits one
C++17 file under 64 KB, with warnings for likely TLE, MLE and overflow.
See docs 4, 4b and 5.
