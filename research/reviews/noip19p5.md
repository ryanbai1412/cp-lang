# noip19p5

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

Cold review: the deque of candidate cuts is a std Deque<Cut<Pos>>, the predecessor table is [Pos.cuts]?Cut<Pos> with no 0 sentinel, and the prefix sums fill a Prefix<Pos, Int> in place so the 4e7 generator case stays within 1024 MB. The generator recurrence threads (b, after) by hand: Gen<_> is not Indexed, so it cannot be zipped with Pos.

## Checklist failures

- `threshold(j)` is recomputed on every deque comparison because caching it as a `[Pos.cuts]Int` trips E0904 at the 256 MB generic profile, even though the problem allows 1024 MB.
- The generator branch reads the six seeds as `(0..B, ...)` and walks `parts` with a manual cursor `k` (`if i > parts[k].p: k += 1`). A run-length iteration over `parts` would remove the cursor.
- Timing under load swings widely: one `--max` run measured 2.55 s against the accepted C++'s 1.13 s, another 0.97 s against 1.10 s. See research/frictions/noip19p5.md.

## Gaps that need the language

- E0904 ignores the problem's own memory limit (repro sent to Core): research/frictions/noip19p5.md.
