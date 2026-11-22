# Learn (`/learn`)

Status: decided in outline (Ryan, 2026-11-21); the chapter and problem
lists are proposed. This replaces the earlier "problems, not lessons"
design, which Ryan rejected after seeing it built: "we just have too
many questions and not enough reading ... i just want to read".

Learn is a course that gets someone off the ground, starting from
another programming language. It is reading first. The guides are not
part of it; they stay in Docs (`50-docs.md`).

## Shape

Learn has two tracks (Ryan: "one is like the reading stuff, the other
is the div 2 a / b practice set"). The four quickstarts
(`20-onboarding.md`), one per language, are the way in.

- **Read**: about eight chapters, read in order, 5 to 10 minutes each.
  A chapter is prose with inline examples. Examples are live editors on
  the real compiler (Run, Trace, C++17). A chapter ends with exactly
  one problem, solved in an editor and judged in the browser. No
  multiple-choice and no predict-the-output questions.
- **Practice**: original problems at the level of a Div 2 A or B:
  mostly implementation, each chosen because solving it teaches a
  specific piece of the language. After AC the page spells that piece
  out and shows the reference solution. Each chapter links to the
  practice problems that use what it covered.

## Read: the chapters

1. *Your first program*: `read` with types, ranges as the statement's
   constraints, `let` and `var`, loops, `print`, running and building.
2. *Arrays and index spaces*: `[0..n]T`, `domain V = 1..=n`, 1-based
   input, grids, cuts and prefix sums.
3. *Optionals instead of sentinels*: `?T`, `??`, `!`, `min=`/`max=`,
   printing "-1 if none".
4. *Functions, structs, enums and `match`*.
5. *Strings and output*: `Str`, `Char`, formatting, separators.
6. *Collections, sorting and searching*: `List`, maps, heaps,
   `sort(by:)`, binary search.
7. *The standard library*: `Mod`, `SegTree`, graphs, `Dsu`; how to find
   what you need.
8. *When it's wrong*: `@!multitest`, `@brute`, `cpl stress`, the trace.

Chapters are MDX in `web/content/learn/`. A fence written
` ```cp run input="..." ` becomes a live editor; the snippet gate runs
every one. Until the 0.3 engine runs bodies, chapters teach forms that
run today and note the 0.3 spelling where it differs.

## Practice: the problems

```
web/content/practice/<id>/
  problem.mdx     # front matter, statement, and "what this teaches"
  starter.cp
  ref.cp          # reference solution; must pass every test
  tests.json      # [{input, expected, sample?}]
```

Front matter: `title`, `level` (A or B), `teaches` (the language
pieces), `chapter` (the chapter it follows). The gate runs `ref.cp`
against every test and compiles the starter. Problems are original;
none copies a judge's statement. Verdicts: AC, WA (first failing test
with a diff), RE (the trap in source terms), CE (the diagnostic). A WA
offers "Trace this test". Progress is kept in `localStorage`; no
accounts.

Target: 30 problems at launch. The track list below is the pool to
draw from; pick the ones that are implementation-heavy and teach a
distinct piece of the language first.

## Problem pool

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

