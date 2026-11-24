// expect: E0201
// rule: 01.7.2
// A label names an enclosing loop; `break 'outer` with no `'outer:` loop is rejected.
for i in 0..3:
    if i == 1: break 'outer // <- here
print(1)
