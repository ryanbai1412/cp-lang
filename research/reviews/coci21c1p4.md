# coci21c1p4

Score: 3/5
Dream: none

Cold review. There is no `dream.cp` and the friction has no `## Dream`, so the port caps at 3. The port follows the header's transform: `Eis` with `turn` and a three-way `plus` (lines 18-20), one radix-3 butterfly per digit (lines 30-37), and the real part of the cubes (line 41).

## Checklist failures

- R1: line 8 says the values are "stored as two Int arrays"; the port stores one `[size]Eis` table (line 22). The header is stale.
- R6: card codes, digits and butterfly slots are raw `Int` (lines 25-27, 30-37).

## Owed rewrites

- Add a `## Dream` to the friction.
- Fix line 8 to say one table of `Eis` values.

## Language gaps

- None; the friction reports none.

## Dream notes

- None written.
