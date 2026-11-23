# `cpl explain`

Status: draft for 0.3.0. The report every implicit choice owes the
contestant (AGENTS.md "nothing secret under the hood"). Its content is
defined by `core/13-lowering.md` §13.6; the LSP shows the same data.

```
$ cpl explain d dist
dist: [V]?Int
  element  8 B, None = Int.MIN
  count    V.count <= 2e5   peak 1.6 MB
$ cpl explain d Ev
Ev  8 B  (Ask: l @0 4 B, r @4 4 B; Add: v @0, r = 0xFFFFFFFF)
?Ev 8 B  None = r 0xFFFFFFFE
$ cpl explain d st
st: SegTree(max, a, act: add)   iterative, no push-down (add commutes)
  nodes 4 B x 2^19 = 2 MB       (max is Selective over 1..=1e9)
```

Sections: types (size, storage order, padding, niches), arrays (width,
peak bytes, `wide_array` and padding notes with fixes), memory total
against the profile, specializations and the laws or bounds that chose
them, remaining checks per build, contexts and generic instances with
output bytes, unsafe-tier sites. `--json` for tools.

Today `cpl explain <file>` prints closure lowering, dropped fields,
views, the reductions the judge build sinks (the list is never built) and
the `Map` and `Set` types stored as hash tables (nothing reads their
order), then lists the sections it cannot fill yet under `Not shown yet:`
(types, arrays and peak memory, specializations, checks); they wait on
the compiler's representation query (plan/QUESTIONS.md tools-3).
