# The course (`/learn/course`)

Status: proposed. The course is problems, not lessons: each page is a
small original contest problem with a short lesson attached, judged in
the browser. It replaces today's 10 lessons (`site/course/*.json`,
*verified*), reusing their topics.

## A problem page

Layout like a judge, two panes:

- **Left**: title, limits (time, memory), statement, input and output
  format with ranges, samples, then a collapsed **Lesson** (what this
  problem teaches, 1 to 3 short runnable snippets, links to the guide
  and reference). After AC: the **Editorial** and the reference
  solution unlock.
- **Right**: the editor with a starter (usually the input already read),
  the test-case panel (samples plus your own cases), and buttons *Run*
  (samples), *Submit* (hidden tests), *Trace* (on the selected case),
  *C++17*.
- **Verdicts**: AC, WA (first failing test, a diff of expected and
  actual), RE (the trap message in source terms), TLE (the interpreter's
  step budget, scaled to the problem's limit), CE (the diagnostic, with
  its fix link). A WA offers "Trace this test".
- **Hints**: up to three, revealed one at a time.
- Progress (solved, attempts, last code) in `localStorage`; no accounts.
  *Export progress* downloads a JSON file; *Import* restores it.

## Problem format

```
web/content/problems/<id>/
  problem.md      # front matter + statement + lesson + editorial
  starter.cp
  ref.cp          # reference solution; must pass every test
  brute.cp        # optional; stress-checked against ref.cp in CI
  tests/01.in 01.out ...   # samples are tests marked `sample: true`
  gen.cp          # optional generator for large tests
```

Front matter:

```
id: grid-bfs
title: Nearest exit
track: foundations
order: 7
tags: [grids, bfs, optionals]
teaches: [Grid, "?T", "??"]
limits: { time_ms: 1000, steps: 5e7 }
status: runs        # runs | checks; set by the gate, not by hand
```

CI checks for every problem: `ref.cp` passes all tests in the
interpreter and as C++ (g++ `-O2`); `brute.cp` agrees with `ref.cp` on
200 generated cases; the starter compiles; the statement's ranges match
the input reads in `ref.cp`. Problems are original; none copies a
judge's statement.

## Tracks and problems

About 64 problems in 8 tracks. Each track ends with one harder
"checkpoint" problem. Problem names and one-line ideas:

### 1. Foundations (8)

1. *Hello judge*: read two numbers, print their sum.
2. *Sum of a list*: ranges as input contracts.
3. *Largest even*: `?Int`, `max=`, `??`.
4. *Several tests*: `@!multitest`.
5. *Counting votes*: maps, `bag`.
6. *Words*: strings, splitting, printing with separators.
7. *Nearest exit*: grids, BFS, `?Int` distances.
8. Checkpoint *Shop queue*: simulation with a deque.

### 2. Index spaces (8)

1. *Prefix sums*: `scan`, `fold(l..=r)`.
2. *Two arrays, two domains*: why `[V]` and `[E]` differ.
3. *Cuts not positions*: splitting an array at cuts.
4. *Sliding window max*: windows over a domain.
5. *Compressed coordinates*: `pieces`.
6. *Matrix walk*: product domains `(R, C)`.
7. *Pairs*: `Pair<V>` unordered pairs.
8. Checkpoint *Painting fences*: difference arrays over cuts.

### 3. Sorting and searching (8)

1. *Sorted order*: `sort`, `sorted(by:)`.
2. *Binary search on answer*: `first_true`.
3. *Last position*: searches that return ends.
4. *Two pointers*: pair counting.
5. *Grouping*: `group(by:, of:)`, `buckets`.
6. *Heap of tasks*: `Heap`.
7. *Intervals*: sweep with events.
8. Checkpoint *Meeting rooms*.

### 4. Dynamic programming (8)

1. *Stairs*: memoized recursion.
2. *Coins*: unbounded knapsack table.
3. *LIS*.
4. *Edit distance*: 2D table over two domains.
5. *Bitmask DP*: `subset`.
6. *DP on intervals*.
7. *Counting mod p*: `Mod<P>`.
8. Checkpoint *Tiling*.

### 5. Graphs (8)

1. *Reachability*: `Graph`, BFS.
2. *Shortest paths*: `dijkstra`, `None` for unreachable.
3. *Components*: `Dsu`.
4. *Topological order*.
5. *All pairs*: `d.closure(over:)`.
6. *Minimum spanning tree*.
7. *Bipartite check*.
8. Checkpoint *Delivery routes*.

### 6. Trees (8)

1. *Rooting a tree*: `Rooted.from_parents`, `try_from_parents`.
2. *Subtree sizes*: post-order.
3. *Depths and buckets*.
4. *LCA*.
5. *Tree DP*: max independent set.
6. *Rerooting*.
7. *Paths with queries*.
8. Checkpoint *Network cables*.

### 7. Range structures (8)

1. *Point update, range sum*: `Fenwick`, `fold(r)`.
2. *Range min*: `SegTree` with `op: min`.
3. *Custom operation*: a user monoid with laws.
4. *Range add, range sum*: `LazySegTree` with an action.
5. *Static RMQ*: `Sparse`.
6. *Offline queries*: Mo's algorithm.
7. *Order statistics*.
8. Checkpoint *Stock prices*.

### 8. Doubt to evidence (8)

These teach the tools, judged on the tool's output where needed.

1. *Write a brute*: `@brute`, then stress finds the bug in a given
   greedy.
2. *Shrink it*: read the shrunk counterexample.
3. *Checker*: a problem with many correct answers, `@checker`.
4. *Interactive guess*: `@!interactive`, `wire.ask`.
5. *Trace the bug*: a planted WA; find the first wrong value in the
   trace.
6. *Explain the cost*: `cpl explain` shows the hidden O(n) copy; fix it.
7. *Sentinel-ended input*: `read_all(until:)`.
8. Checkpoint *Guess the permutation* (interactive).

### Practice set

Beyond the tracks, a filterable list (by tag and difficulty) of all
problems, plus links into the real problems gallery for "solve the real
thing on the judge".

## Content plan

Tracks 1 to 3 at phase 3 start, the rest as content work afterwards. A
problem lands only when its `ref.cp` runs; problems whose natural
solution needs a 0.3 form that doesn't run yet wait, listed in
`web/content/problems/WAITING.md` with the missing form.
