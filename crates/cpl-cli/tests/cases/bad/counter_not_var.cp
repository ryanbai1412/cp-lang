// error: `seen` is a `let`, so it can't be assigned
let n: 2..=9 = read()
let a: [n]Int = read()
let seen = counts(a)
seen[3] += 1
