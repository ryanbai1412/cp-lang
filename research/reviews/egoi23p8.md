# egoi23p8

Score: 2/5
Dream: none

Cold review of `port.cp` (no `dream.cp`, no friction `## Dream`, so capped at 3). Run 1 sends `s / 2` as a composition through `unrank` and run 2 reads it back with `rank` (lines 61, 78), as the header says.

## Checklist failures
- idiomatic-code/R4: the protocol shape is in `PARTS = 9` and `TOTAL = 11` (lines 14-15) but restated as `print(16)` (line 52), `n - 13` (line 55), `n - 12` (line 75), labels `2 + i`, `11 +` and `<= 10` (lines 65, 69, 73-74, 83). Not in the friction.
- idiomatic-code/R2: the friction describes a pure `compositions(parts, total)` list and whole-list ranking; the port instead has hand-written `ways`/`unrank`/`rank` (lines 18-45), so the report does not describe the code.

## Owed rewrites
- Derive K, the reserved house counts and the label bands from `PARTS` and `TOTAL`.
- Make the friction describe the rank/unrank code (or go back to the listed compositions it describes).

## Language gaps
None shown.

## Dream notes
No dream.
