# Friction report: btoi17p3

The port now sorts with the method value, `plan.sorted(by: tree.pos)`, as the dream wrote it. Core's method-value fix makes `tree.pos` borrow `tree` rather than copy it per key call. `scripts/corpus.py verify --only max btoi17p3` reports 0 findings.

No language gap.

## Measurements

gen.py --max: port 0.02 s, accepted C++ 0.10 s, same output.
