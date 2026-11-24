# btoi20p3 review

Score: 2/5
Dream: 4/5

Cold review against the idiomatic checklist, on main with this report. The port checks and lints
clean, agrees with `orig.cpp` on `--max`, and meets the 1.1x time and RSS bar (1.04x, 0.85x).

- R1: header complete; the monotonicity of `limit` is stated, not argued (one line on why removing
  more of the prefix can't make an odd cycle appear would close it).
- R3/R4: a hand rollback parity DSU with three parallel history lists, where std has `ParityDsu`
  and `undoing` (friction: blocked on the dream's 1.16x RSS).
- R5: `m + 1` as "never" in `limit` and `opt`; the reads are now bounded.
- R6: `E` and `V` are distinct domains, but `dnc` takes four raw `Int` bounds and indexes with
  `E(i)`, so the recursion leaves the domain.
- R7: `odd_cycle` and the history lists are globals threaded through `add_edge`/`rollback`.

Dream: one `ring.minimal` over an undoing parity DSU states the editorial's two-pointer exactly; it
loses a point for the RSS it costs today.
