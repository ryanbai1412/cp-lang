// expect: E0305
// rule: 12.4.5
print([1, 2, 3].filter(x => x > 1)[0]) // <- here
