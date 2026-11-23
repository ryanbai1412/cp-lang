// expect: E0305 at 3
// rule: 07.4.5
// A filtered view keeps keys but skips some, so it has no `[]`; `.nth(k)` reads by position (decision 388).
let a = [1, 2, 3]
print(a.filter(x => x > 1)[0])
