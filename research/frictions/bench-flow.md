# Friction report: bench/flow (generated C++ costs found profiling `std.flow`)

Found while profiling `FlowGraph.max_flow` against matched C++ (`bench/flow/dinic`,
5000 vertices, 200000 arcs, best of 21). The solve is not the gap: alone it is about
28 ms in CP Lang and 39 ms in the C++. Building the network is: 47 ms against 20 ms.
Each section below is one piece of that, measured by editing the generated C++.

## `xs.extend(ys[a..])` copies the slice into a temporary list first

- **Where:** `cap0.extend(left[cap0.len..])`, tried in `FlowGraph` to copy initial
  capacities once per solve instead of pushing them per `add`.
- **What felt wrong:** the obvious spelling emits `auto t0 = cpl::to_list(cpl::list_slice(...));
  t1.insert(t1.end(), t0.begin(), t0.end());`, a 3.2 MB allocation and two copies for
  400000 arcs. With a direct `insert` from the slice the change saves 2-5 ms of 76 ms;
  through the temporary it saves nothing, so the library keeps pushing `cap0` per `add`.
- **Goal it hurts:** performance
- **Ideal code:** `cap0.extend(left[cap0.len..])`, as written.
- **General fix:** `extend` (and `push` of a range in general) takes a view and emits
  `insert(end, begin, end)` from it, with no materialized list.
- **Severity:** medium
- **Status:** open
- **Area:** codegen

## `FlowGraph.add` is not inlined into the build loop

- **Where:** `g.add(V(u), V(v), next(1000000) + 1)` in `bench/flow/dinic.cp`.
- **What felt wrong:** `FlowGraph_D0_V_8_add` is a separate 14% of the profile; the head
  push for `a.rev` goes through an immediately-called lambda (`[&]() -> void { auto t0 =
  m1_Arc_rev(a); self.head[v].push_back(t0); }();`). With `cap0` and the generator's
  `%` matched, the build is still 29 ms against 20 ms for the C++, whose `add` is inline.
- **Goal it hurts:** performance
- **Ideal code:** the same; the call should cost what the C++ `add` costs.
- **General fix:** emit small library methods `inline` and pass call results straight to
  `push_back` instead of wrapping the statement in a lambda.
- **Severity:** medium
- **Status:** open
- **Area:** codegen

## Every program runs on a big-stack thread

- **Where:** the generated `main` tries `pthread_create` with a 1 GB stack, halving down
  to 8 MB, before running the program.
- **What felt wrong:** an empty program takes 2.4 ms against 1.1 ms for an empty C++
  `main`; on `dinic` removing the thread saves about 3 ms of 76 ms (`malloc` in a
  non-main thread uses a separate arena too).
- **Goal it hurts:** performance
- **Ideal code:** nothing to write; the deep-recursion guarantee should cost nothing
  when the program never recurses deeply.
- **General fix:** start the thread only for programs whose call graph has recursion, or
  raise the main thread's stack limit with `setrlimit` and re-exec where the judge allows.
- **Severity:** low
- **Status:** open
- **Area:** codegen

## `for j in 1..=m` lowers to a flag loop the optimizer can't vectorize

- **Where:** the three `for j in 0..=m` / `1..=m` scans in `hungarian` (`std.flow`), the
  `bench/flow/hungarian` row (1500 x 1500).
- **What felt wrong:** an inclusive range emits
  `for (ll j = 1, j_go = j <= m; j_go; j_go = j < m && (++j, true))`, safe when
  `m == Int.MAX` but opaque to gcc's loop passes. Rewriting only those headers to
  `for (ll j = 1; j <= m; ++j)` takes the program from 373 ms to 270 ms, under the
  matched C++'s 278 ms. Spelling it `1..m + 1` in the library would hide the cost, so the
  library keeps `..=`.
- **Goal it hurts:** performance
- **Ideal code:** `for j in 1..=m:`, as written.
- **General fix:** emit the plain `<=` loop when the bound's type proves `m < Int.MAX`
  (any `Int` from `.len`, `.K` or a range type does), and the flag loop only otherwise.
  `..=` is in 28 places in `stdlib/flow.cp` and about 1,200 port files.
- **Severity:** high
- **Status:** fixed: the codegen emits `j <= m; j++` when `m` is a length, a domain size, a
  bounded value or less than one of those, set once; the `hungarian` row goes from 1.33x to
  1.02x matched C++.
- **Area:** codegen

## `unchecked:` keeps domain and range checks

- **Where:** `R.nth(j - 1)` and `p[0] = i` inside `hungarian`'s `unchecked:` block.
- **What felt wrong:** `unchecked:` drops `cpl::at`, but `R.nth(j - 1)` still emits
  `cpl::pos(j - 1, R_len)` in the 2.25M-step inner loop, and a store into a
  `List<0..2_147_483_648>` still emits `check_range`. Dropping the `pos` call alone saves
  39 ms of 373 ms.
- **Goal it hurts:** performance
- **Ideal code:** the same `unchecked:` block covering every check it lexically contains.
- **General fix:** decide whether `unchecked:` means index checks only or every
  bounds-style check (index, domain `nth`, range-typed store), record it, and have the
  codegen follow it.
- **Severity:** medium
- **Status:** fixed per the amendment to decision 43.8: the C++ drops the `pos` check of
  `V.nth(i)` and `V(label)` inside `unchecked:`, `cpl run` keeps it, and range-typed stores
  stay checked everywhere. With both fixes `hungarian` is 0.93x matched C++.
- **Area:** codegen

## `?(0..N)` is stored as an 8-byte optional

- **Where:** `alternating_layers`'s `var level: [L]?(0..2_147_483_648)` in `max_matching`
  (Hopcroft-Karp, `bench/flow/matching`, 100000 + 100000 vertices).
- **What felt wrong:** `[L]?V` over a domain gets the 4-byte `Niche<int, int, -1>`, but
  an optional of a range that fits 31 bits is still `Niche<ll, ll, INT64_MIN>`. Using the
  4-byte niche for `level` takes the program from 129 ms to 121 ms (matched C++ 107 ms).
- **Goal it hurts:** performance
- **Ideal code:** `var level: [L]?(0..2_147_483_648) = none`, as written now.
- **General fix:** pick the optional's storage from the payload's range, as plain range
  types already do.
- **Severity:** medium
- **Status:** fixed: `?lo..hi` inside 31 bits is `cpl::OptI32` in lists, arrays and
  their returns, and a store into it is range-checked (it was not, even at 8 bytes).
  `matching` did not move (126 to 131 ms against 101 to 110 ms over runs): the two
  entries below take what the 4 bytes gave.
- **Area:** codegen

## A narrow optional is widened to compare it

- **Where:** `level[w] == level[u]! + 1` in `max_matching`'s search.
- **What felt wrong:** each read of a `cpl::OptI32` element becomes `cpl::OptInt(...)`
  before `==` and `!`, a select per read in the hot loop. Comparing the raw 4-byte
  values instead takes `matching` from 129.6 to 124.9 ms (perf stat, 8 runs).
- **Goal it hurts:** performance
- **Ideal code:** as written.
- **General fix:** compare two optionals of one payload at the narrower storage, and
  read `x!` from the narrow value, without building the wide optional.
- **Severity:** low
- **Status:** open
- **Area:** codegen

## A domain with constant bounds has a runtime length

- **Where:** `domain L = 0..100000` in `bench/flow/matching`; every static domain.
- **What felt wrong:** `L_len` is a global set at the start of `solve`, so `seed % L.K`
  is a division by a variable and every `u < L_len` loop reloads it. Emitting
  `constexpr ll L_len = 100000` instead takes `matching` from 126.4 to 120.7 ms
  (matched C++ 108.0 ms, whose `N` is a constant).
- **Goal it hurts:** performance
- **Ideal code:** as written.
- **General fix:** a domain whose bounds are constants emits `constexpr` bounds.
- **Severity:** medium
- **Status:** fixed: a domain defined once from literals, and locals or globals set
  once to them, gets `constexpr ll D_lo, D_len` and no per-call `DomGuard`. Whole
  `bench/flow` table, best of two runs of best of 5, before then after: `matching`
  1.26x to 1.15x, `hungarian` 0.94x to 0.80x, `dinic` 1.18x to 1.11x, median 1.10x to
  1.07x, max 1.26x to 1.17x (noisy machine: rows moved up to 0.04x between runs).
  Corpus: 78 of 1654 emitted files change, all shrink (largest cut 272 B).
- **Area:** codegen

## Measurements

| | CP Lang | C++ |
|---|---|---|
| `dinic`, whole program | 75.8 ms | 59.0 ms (64.8 ms keeping `cap0`) |
| network build only | 47.4 ms | 20.3 ms |
| empty program | 2.4 ms | 1.1 ms |
| `hungarian`, as emitted | 373 ms | 278 ms |
| `hungarian`, plain `<=` loops | 270 ms | |
| `matching` (Hopcroft-Karp), as emitted | 129 ms | 107 ms |
| `matching`, 4-byte `level` (hand-edited) | 121 ms | |
| `matching`, `cpl::OptI32` as emitted | 126 to 131 ms | 101 to 110 ms |

Emitted C++ for `bench/flow/dinic.cp`: 14,119 B.
