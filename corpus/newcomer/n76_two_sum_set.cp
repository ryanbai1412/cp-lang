let (n, x): (1..=200000, 1..=1000000000) = read()
let a: [0..n]Int = read()
var seen: Set<Int> = Set()
var found = false  // expect: missing flag-for-first-match
for v in a:  // expect: search_loop
    if (x - v) in seen:
        found = true
        break
    seen.add(v)
print(yes(found))
