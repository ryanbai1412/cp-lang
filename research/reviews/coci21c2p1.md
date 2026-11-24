# coci21c2p1

Score: 3/5
Dream: none

Cold review. There is no `dream.cp` and the friction has no `## Dream`, so the port caps at 3. The port is the header's counter per depth: match the command to its depth (lines 16-19), bump it, zero the deeper ones and print the prefix (lines 20-22).

## Checklist failures

- No dream.
- R4, minor: the deepest level `3` is written in the table type (line 12) and again in the reset loop (line 21); a `domain Depth = 1..=3` would hold it once.

## Owed rewrites

- Add a `## Dream` to the friction.
- `domain Depth = 1..=3`, then `var number: [Depth]Int` and reset `for d in Depth if d.ord > depth`.

## Language gaps

- None; the friction reports none.

## Dream notes

- None written.
