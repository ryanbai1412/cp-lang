# ioi04p2

Score: 3/5
Dream: 3/5

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 3/5 (capped: the dream is under 4), dream 3/5, from the friction's `## Dream`. Two `[Coord]Int` arrays of least walks, each updated per junction by "keep the free coordinate" or "turn at the meeting point" (lines 20-31): the header's DP, O(n * 2001).
- Distances go through `.ord` (lines 25, 27-28); in the friction.
- Lines 26-28 update both arrays in a `for v in Coord:` loop; `on_x = [Coord] v => min(on_x[v] + dx, ...)` says each in one line and runs today. The friction marks its old `?Int` entry fixed and notes this rewrite.
- Comments say "lighthouse" (line 17) where the header and statement say junction.

## Dream notes

- The dream elides most of the program with `...` and leaves `min_by((v, w) => ...)` undefined; a full dream.cp is owed.
