# Friction report: aac5p4

spec.cp printed the fewest moves M and then 1..=M, copied from the accepted
C++, which only earns the partial score for M; it now builds a valid move
sequence (the port's construction) and carries the port's `@check`.

## The corpus suite ignores the problem's checker

- **Severity:** medium
- **Status:** open (routed to spec, owner of `crates/cpl-conform`)
- **Goal it hurts:** doubt to evidence
- **Area:** tools (`cpl-conform corpus`)
- **What happened:** the moves are not unique, but `cpl-conform corpus`
  compares tokens with `tests/*.ans`, so spec.cp passes only by printing the
  same sequence as port.cp, and any other valid construction reads `wrong`.
  The file's `@check` (and `checker.py`) already accept every valid answer
  through `cpl judge`.
- **General fix:** when the problem has a `@check` or `checker.py`, the
  corpus suite judges each output with it instead of the exact compare.
