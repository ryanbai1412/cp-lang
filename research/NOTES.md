# Research notes for decisions 1-4 (paused; resume after the compiler series)

Corpus: 3462 C++ files in ../corpus. Benchmarks: bench/checks.cpp, bench/gen.cpp
(g++ 11.4 -O2, best of 5, pinned core).

## 1. Borrows held by structs
- 2421 `auto f = [&](...)` lambdas in 1274 files (37%). 30 use a self param
  (recursion). 24 are passed as arguments and 7 are returned; the rest are only
  called locally. Nested `fn` / `mut fn` covers nearly all of them.
- Struct fields that point into outside data (top-level fields only): pointer
  fields in 45 files (1.3%), reference fields in 21 files (0.6%). Examples:
  `Ans.arr: vector<P>&`, `Tree.st: ST*`, `Block.arr: int*`.
  Node self-pointers (treaps) are separate and handled by pools.

## 2. Generators
- Pull state machine (struct + inline next()) == hand loop for runs (69 vs 74 ms),
  0 calls in the asm. zip of two pull generators: 64 vs 59-61 ms hand.
  Outer push + inner pull: 57 ms.
- Recursive preorder: push lowering (recursive callback) 80 vs 81 ms hand
  recursion. Explicit stack (pull): 87 ms, about 8% slower than recursion.
- Implication: non-recursive generators can lower to pull structs at ~0 cost,
  which allows zip and possibly storing them. Recursive ones lower push-style.

## 3. Index arithmetic (x[i +- 1])
- other 27.4% of files, dp 12.2%, neighbour-in-input 12.5%, prefix 12.2%,
  index through a map array (tin/pos/id...) 1.9%. Top arrays: dp, arr, psum.

## 4. Checks (ms, no checks / idx / overflow / both)
- prefix 47/50/49/50, segtree 84/92/88/95, bfs ~240 (noise), knap 68/101/101/126,
  matmul 25/34/27/50. Checks hurt where they block vectorization (knap, matmul).
