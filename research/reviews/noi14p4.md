# noi14p4

Score: 3/5
Dream: none

Port 3/5, dream none. The friction report has no `## Dream` section and there is no `dream.cp`, so the port is capped at 3. Border depths over the prefix function and a second KMP pointer `half` capped at half the prefix (lines 16-25), as the header says.

## Checklist failures

- The friction says the depths are `Rooted.from_parents` over the prefix function, but the port steps them by hand: `for j in 1..=s.len: depth.push(depth[pi[j - 1]] + 1)` (line 17). The friction is corrected.
- The two `while` loops on `half` (lines 22, 24) are kept explicit, as the friction explains.

## Owed rewrites

- Add a `## Dream`.

## Language gaps

None.

## Dream notes

None.
