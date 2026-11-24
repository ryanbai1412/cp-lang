# cco02p3

Score: 4/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header covers the protocol.
- R2: depth-first search with the robot, undoing each step: the editorial.
- R3: the way back (`right(); right(); forward(); left()`, L43-46) is written out (reported).
- R11: the report's `read(letters: "NESW")` ideal had no run; added (E0310).

## Owed rewrites

None.

## Language gaps

- Scoped moves undone on exit (in the report).
- Typed `Dir4` read (in the report; now with a `cpl run`).

## Dream notes

`with forward() as moved` is the right ask.
