# noi04p1

Score: 4/5
Dream: 4/5

Port 4/5, dream 4/5 (the friction: the port is the dream). One online pass: base salaries in a `SortedBag`, a global `raise`, `drain(while: ...)` counts who leaves on a cut (line 23), and `nth` answers the k-th largest (line 24). Every operation is the editorial's.

## Checklist failures

- idiomatic-code/R5: the argument is read as `0..=1e9` for every command (line 17), so `F 0` would reach `staff.nth(staff.len)!` and abort; the statement's k for `F` is at least 1, which the read does not say.

## Owed rewrites

- Read the argument per command (a tagged `enum Cmd` with `F(k: 1..=1e5)`), which also removes `_ =>` standing for `F` on line 24.

## Language gaps

None beyond the friction.

## Dream notes

The port; a typed command read would make it 5.
