# atcoder_dp_t

Score: 4/5
Dream: 4/5

Cold review against the idiomatic checklist.

- R1: complete (limits, original, Bounds).
- R2: `ways[r]` over `Rank` with one running sum per sign is the editorial's O(n^2).
- R5/R6: `Rank = 0..n` for relative ranks and `Gap = 1..n` for signs are distinct spaces;
  `Rank(g.label)` converts between them explicitly.
- R8/R13: `const P` is used once; `Mod<1_000_000_007>` is the corpus spelling.
- R10: 0.79x.

## Checklist failures

R8 (modulus constant).

## Owed rewrites

Inline the modulus in both port and dream.

## Language gaps

None.
