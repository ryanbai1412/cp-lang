# ucup5s05e friction report

## Missing stdlib: `k`-subsets / `combinations(k)` on sequences in `std.seq`
- **Where:** `proto/ports/ecnerwala/dream/ucup5s05e.cp` line 43 (`proto/ports/ecnerwala/ucup5s05e.cp` lines 43–47).
- **What felt wrong:** Testing every 3-element subset of a small candidate list requires three nested index loops (`for i in 0..cands.len: for j in i + 1..cands.len: for k in j + 1..cands.len:`) with an early return.
- **Goal it hurts:** express.
- **Ideal code:**
  ```cp
  if let tri = cands.combinations(3).first(t => in_tri(pts[t[0]], pts[t[1]], pts[t[2]], c)):
      return tri
  ```
- **General fix:** Add `combinations(k)` (or a `triples()` / `combinations(k: Int)` iterator/helper) to `std.seq` that yields `k`-element combinations in index order without duplicating nested index loops.
- **Severity:** low.
