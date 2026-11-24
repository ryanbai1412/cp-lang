# noi15p2

Score: 3/5
Dream: none

Port 3/5, dream none. The friction report has no `## Dream` section and there is no `dream.cp`, so the port is capped at 3. Install sets a root path of heavy chains and uninstall clears a subtree range, both on a `SegTree(+, act: assign)` that counts before assigning (lines 21-34).

## Checklist failures

- `boss` is filled by a loop (lines 17-18) to shift the parent list onto `Pkg`.
- The commands are `if cmd == "install"` with `else` for uninstall (lines 31-34).

## Owed rewrites

- Add a `## Dream`; a tagged command read.

## Language gaps

None.

## Dream notes

None.
