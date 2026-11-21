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

## Measurements

| | CP Lang | C++ |
|---|---|---|
| `dinic`, whole program | 75.8 ms | 59.0 ms (64.8 ms keeping `cap0`) |
| network build only | 47.4 ms | 20.3 ms |
| empty program | 2.4 ms | 1.1 ms |

Emitted C++ for `bench/flow/dinic.cp`: 14,119 B.
