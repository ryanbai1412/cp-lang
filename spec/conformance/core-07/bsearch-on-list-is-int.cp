// expect: ok, output "2 5"
// rule: 07.12.5
// On a `List`, `bsearch` gives an `Int` cut usable as an index (round 26, H12).
let a = [1, 3, 5, 7]
let j = a.bsearch(x => x >= 5)
print(j, a[j])
