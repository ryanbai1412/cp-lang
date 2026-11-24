# atcoder_dp_i

Score: 3/5
Dream: 5/5

Cold review of `port.cp` against the friction's `## Dream` (there is no `dream.cp`), without the
C++. One `[Heads]F64` table over `Heads = 0..=n`, updated in place going down with `h.next`, then
the majority sum: the editorial.

## Checklist failures
None in the code. The only cap is the missing `dream.cp`: a port with no dream scores at most 3.
The dream differs only in types today's language lacks (`Odd<1..=2999>`, `[n]0.0<..<1.0`).

## Owed rewrites
- Commit a `dream.cp`. It cannot be the friction's dream verbatim, since `corpus.py verify`
  builds the dream and both types fail (E0204, E0105).

## Language gaps
- F64 range types (`0.0<..<1.0`): in the friction, with a run.
- Parity refinement (`Odd<...>`): in the friction, with a run.

## Dream notes
Reads like the editorial, with every statement bound a type.
