# cco05p6

Score: 3/5
Dream: 3/5

The windows are a domain, the stack is `Win.collect()` bottom to top, and a click is `stack.keys.rfind(...)`, `remove`, `push`: the brute force the statement invites, with no hand-shifted tail.

## Checklist failures

- idiomatic-code/R5: the frame and click coordinates are read as `Int`, though the statement bounds them by the screen (<= 1e4); the `// Bounds:` line names only n and m.

## Owed rewrites

- Type the coordinates (`0..=10_000`, or a `Coord` alias read into `frame` and `clicks`) and add them to `// Bounds:`.
- `hit` could read `f.left <= x <= f.right and f.bottom <= y <= f.top`; chained comparisons type-check under 0.2.

## Language gaps

- `List.remove(i)` is positional but reads like a value: research/frictions/cco05p6.md.
- The dream's `stack.move_to_back(at: i)` (remove and push as one call) is not reported; it belongs with the `remove(at:)` gap.

## Dream notes

- `for (x, y) in (read(): [m](Int, Int))` is not the canonical `read<[m](Int, Int)>()` (idiomatic-code/R8, caps at 3).
- The closure `f` spells `frame[w]` four times where a `let f = frame[w]` (as the port has) says it once.
- A dream under 4 is sent back and caps the port at 3 (research/reviews/README.md); the port itself reads at 4.
