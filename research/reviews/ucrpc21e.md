# ucrpc21e

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review of `port.cp`, without the C++. The header derives the key fact (each time unit
independently takes the better mouth, so the answer is the sum over time of the larger rate)
and the endpoint-event sweep, with its cost. `domain Mouth = [2]` and `rate: [Mouth]Int` keep
the two mouths a domain, and `score += (c.at - last) * rate.max()!` is the editorial's sum.
A port with no dream scores at most 3.

## Checklist failures
- No dream: neither `dream.cp` nor a friction `## Dream` exists (porting.md step 2).
- idiomatic-code/R2: the events are built by pushing in a nested loop (L17-21) and sorted in
  place, because `flat_map` once lost the tuple element type (the friction). That gap is gone:
  `List(Mouth.flat_map(mouth => windows[mouth].flat_map(w => [(at: w.a, mouth:, d: worth(w)),
  (at: w.b + 1, mouth:, d: -worth(w))]))).sorted()` now checks and prints the events in order
  under a fresh `cpl run` (2026-11-24).

## Owed rewrites
- Write the dream (the friction's `## Dream`, or `dream.cp`).
- Build `changes` with the `flat_map ... .sorted()` above; the friction's entry is fixed.

## Language gaps
- None open: the friction's `flat_map` element-type gap now runs (status added there).

## Dream notes
No dream to score; the port is judged on its own and capped at 3.
