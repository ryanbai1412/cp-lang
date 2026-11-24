# noi07p6

Score: 3/5
Dream: none

Port 3/5, dream none. The friction report has no `## Dream` section and there is no `dream.cp`, so the port is capped at 3. The need is a std `reroot` with a top-two monoid (lines 22-28), then the plan replays the header's argument from the best root.

## Checklist failures

- `sweep` (lines 38-51) folds the first lighter child into the loop as `if i == 1 and not guarded: sweep(cs[0], ...)` and handles one child in a separate branch (line 50), so which child is cleared when is hard to follow.
- `fn land` (line 34) wraps one `push` while `B`, `M` are pushed inline (lines 44, 47, 51).
- The friction's `max_k` ordering problem no longer applies to the port, which uses the `top2` monoid; the friction does not say so.

## Owed rewrites

- Add a `## Dream` to the friction; split `sweep` into "clear the lighter children" and "step into the heaviest" as the header words it.

## Language gaps

None beyond the friction.

## Dream notes

None.
