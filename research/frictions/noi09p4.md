# Friction Report: noi09p4

`corpus/problems/noi09p4/spec.cp` is spec-valid but does not run: every remaining failure is a
compiler gap with a product domain `Cell = (Row, Col)`. Repros are in the compiler worklog
(2026-11-23, from idioms, `noi09p4`).

## 1. A product domain as a graph's vertex
- **Where:** `peel(protects)` and `max_closure(before: (p: Cell) => ..., profit: ...)`.
- **What felt wrong:** a fn, lambda or `[Cell]List<Cell>` over a product domain is not usable as a
  `Next<Cell>`: `peel(fn)` hits the known `G.Arc` E0204; `max_closure` fails inside its body
  (`(Row, Col)` has no `ord`, or the lambda is typed `Int`); `[Cell]List<Cell>` binds `V` to
  `List<Cell>`. The only form that runs flattens cells by hand (`domain Flat = [rows * cols]`,
  `Flat.nth(p.ord)`), the C++ idiom the domain exists to avoid.
- **Goal it hurts:** express, coherence.
- **Ideal code:** the current spec.cp.
- **General fix:** a product domain is a domain everywhere a generic `V` goes (decision 159), in
  std bodies too; helps every grid graph port (bfs/components/flow over cells).
- **Severity:** blocker.

## 2. Reading a product position
- **Where:** `read<[Cell](score: ..., protects: List<Cell>)>()`.
- **What felt wrong:** a `Cell` reads one token as a flat ordinal instead of `(row, col)`, so
  the input `r c` pairs are misread silently (or trap as out of range).
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** the current spec.cp.
- **General fix:** a product position reads as its tuple of factors (R04.2.10, R10.2.1).
- **Severity:** high (silent wrong input).

## 3. Inline product of counted factors
- **Where:** the earlier `domain Cell = ([rows], [cols])`.
- **What felt wrong:** E0501; spec.cp names `Row` and `Col` instead (already routed, compiler
  worklog, coci21c2p4).
- **Severity:** low.

Measurements: spec.cp does not run; the flattened variant matches AC 5144326 on 8/8 tests and
60/60 gen.py inputs.
