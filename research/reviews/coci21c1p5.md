# coci21c1p5

Score: 3/5
Dream: 3/5

Cold review against `dream.cp`; the port is the dream with a `Level` domain and the levels copied into a `[Level]List<Pos>` (line 28, reported). The header (lines 6-11) derives the levels and the throw-away rules; the loop (lines 32-47) is that greedy.

## Checklist failures

- R4: the "goes on the chain" step is written twice, `chain.push(at)` then `used[h] += 1` (lines 43-44 and 46-47), once for an empty chain and once after both tests fail. Not a language gap.
- R6, minor: `used` counts into `level[h]` by raw `Int` (lines 29, 37-38).

## Owed rewrites

- One push branch: test `chain.last` against `at` with `is_some_and` for the two throw-away cases, then a single `else: chain.push(at); used[h] += 1`.
- The list-to-table friction quotes no run of `on_level.into([Level])` (R11); add it.

## Language gaps

- An owned `List<T>` to `[D]T` conversion (line 28 copies every level): in the friction.
- E0603 on a nested-list read then push: in the friction, with a repro.

## Dream notes

- The header comment is the editorial, but the code is the C++ loop: `next: [0..len]Int` over a raw range and the same duplicated push branch (lines 41-45). The dream should name `Level` and push in one place.
