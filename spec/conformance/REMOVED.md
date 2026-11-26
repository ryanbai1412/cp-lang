# Removed cases

A conformance case or new-pipeline test leaves only with a line here naming it and why
(`scripts/debt-ratchet.py`, `next_cases_removed`).

- `crates/cpl-next/tests/programs/a_own_19_generic_return_moves.cp`: renamed to
  `a_own_19_generic_return_lends.cp`. It expected `id(b)` to move `b`, but R05.5.3 says a
  generic fn's result that holds its argument lends it; today's `cpl` agrees and prints `2 1`.
