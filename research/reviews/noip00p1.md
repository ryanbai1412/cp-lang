# noip00p1

Score: 2/5
Dream: 4/5

Cold review on main 6a989896d: port, dream and friction report, not the C++. Euclidean division by a negative base is the header's argument and the dream's line. But the port prints nothing for x = 0, which the bounds allow (`x: -32768..=32767`): `echo "0 -2" | cpl run port.cp` prints an empty line, since `while left != 0` never runs.

## Checklist failures

- Lines 13-18 print an empty digit list for `x = 0`: a wrong answer on a valid input (correctness first).

## Owed rewrites

- Print `0` for `x = 0`, and add that case to tests.toml.

## Language gaps

- `Char.digit` and `Int.digits(base:)` for negative bases: friction 1 (run added: `error[E0201]: no value named `Char` here`; `5.digits(base: -2)` fails `assertion failed: self >= 0 and base >= 2`).
