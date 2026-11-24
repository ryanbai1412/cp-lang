// expect: R0001 (checked)
// rule: 07.3.5
// A range's slice must lie inside it: `(10..20)[5..12]` starts before the range does.
print((10..20)[5..12].len) // <- here
