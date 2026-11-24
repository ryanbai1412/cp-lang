# ceoi09p3

Score: 4/5
Dream: 4/5

Cold review against the friction's `## Dream`. The interval DP is the header's recurrence: compressed heights (line 18), `tallest` from `height.last_le(area / w)` (lines 20-22), and `@memo(array) cover` over typed `P.cuts` and `H` (lines 24-32), with the split as one `min()` comprehension.

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- The friction's two claims (`sorted()` loses the table type; `[0] + [p.y for p in pts]` fails) quote a type pair but no command; add the `cpl run` lines.

## Language gaps

- `read<[P]Pt>().sorted()` returns a `List`, so the port sorts a `var` table in place (lines 16-17): in the friction.
- `[0] + [p.y for p in pts]` fails, so line 18 writes `.map(...).collect()`: in the friction.

## Dream notes

- Two lines over the port (a sorted typed read, compress without `.collect()`), judged as the port with them; that reads like the editorial.
