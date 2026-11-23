// expect: E0201 at 3
// rule: 01.7.2
// A label names an enclosing loop; `break 'outer` with no `'outer:` loop is rejected.
for i in 0..3:
    if i == 1: break 'outer
print(1)
