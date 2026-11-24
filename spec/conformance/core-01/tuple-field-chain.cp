// expect: ok, output "2 1"
// gap: compiler 2026-11-24T05Z
// rule: 01.5.1
// Postfix `.0` chains (R01.6): `t.0.1` is field 1 of field 0, not `t` followed by the float `0.1`, as in Rust.
let t = ((1, 2), 3)
print(t.0.1, (t.0).0)
