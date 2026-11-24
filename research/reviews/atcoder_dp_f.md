# atcoder_dp_f

Score: 4/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp`, without the C++. `common[i, j]` over suffixes, the
equal-head lemma in the header, and the walk that rebuilds one LCS are all the editorial's.
`[I, J]0..=3000` states the length bound.

## Checklist failures
- idiomatic-code/R1 (minor): no `// Bounds:` line. `Str<1..=3000>` carries the bounds in the
  type, but the header doesn't state them.

## Owed rewrites
- A `// Bounds: a, b: 1..=3000 characters` header line.

## Language gaps
None.

## Dream notes
The dream's table is `[I, J]Int` and should take the port's `0..=3000`. Noted in the friction.
