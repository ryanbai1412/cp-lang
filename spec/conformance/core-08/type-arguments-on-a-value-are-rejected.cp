// expect: E0302
// rule: 08.2.5
// Only a generic fn, type or const takes type arguments.
let n = 3
let m = n<Int> // <- here
print(m)
