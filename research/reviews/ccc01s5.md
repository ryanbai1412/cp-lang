# ccc01s5

Score: 2/5
Dream: 4/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. The header explains
the search: keep only the spare part of the row that is ahead, drop a pick whose rows disagree,
stop at m - 1 words. The port keeps the answer in a global stack pushed and popped around each
try (lines 17, 24, 29), and `search` returns only `Bool`.

## Checklist failures
- idiomatic-code/R7/R2: the undo stack `picked` (lines 17-29). The friction proposes
  `Pair.find_map` and list prepend as the fix, but std already has both: `first_some` on any
  sequence (std/seq.cp, used on a domain as `T.first_some` in other ports) and `[p] + l`. The
  friction calls this a missing feature; it is bookkeeping the language already removes.

## Owed rewrites
- `fn search(ahead: Str, top: Bool, left: Int) -> ?List<Pair>` with
  `Pair.first_some(p => ...)`, as the dream has it. Correct the friction.

## Language gaps
None; `first_some` and list concatenation exist.

## Dream notes
Reads like the editorial and returns the witness. `Pair.find_map` should be the std
`Pair.first_some`, and the one-line `if let ...: ...; for ...` output branch is cramped.
