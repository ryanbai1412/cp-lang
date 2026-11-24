# cco17p4

Score: 3/5
Dream: 3/5

The header derives the tallest-first subset sum and the code is that DP: a `Bitset<Volume>` grown by one `union_shifted` per distinct earlier height, with an `@alt` brute over every order beside it.

## Checklist failures

None beyond the reported gaps.

## Owed rewrites

- `height.sorted().rev.collect()` (line 19): under 0.2, `sorted(desc: true)` is rejected with `error[E0303]: a sort puts the largest first with a descending key, not desc:`, so the friction's ideal is itself a removed spelling; sort by a descending key instead.

## Language gaps

- The `before` snapshot before the shifts: research/frictions/cco17p4.md.

## Dream notes

- `height.sorted(desc: true)` is the spelling the compiler names as removed (idiomatic-code/R8, caps at 3). With a descending key, and `tall[1..i].distinct` for the heights, the dream would be a 4. No header.
- A dream under 4 is sent back and caps the port at 3 (research/reviews/README.md); the port itself reads at 4.
