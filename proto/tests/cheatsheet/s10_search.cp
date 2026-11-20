let n: 1..=10 = read()
let a: [0..n]Int = read()
let lo = bsearch_first(0..n, i => a[i] >= 7)
let hi = bsearch_last(0..n, i => a[i] <= 3)
let no = bsearch_first(0..n, i => a[i] > 100)
let (i, w) = bsearch_first_some(0..10, i => if i * i >= 10: i * i else: none)!
print(lo, hi, no, i, w, a.first_ge(2), a.last_le(8), a.first_ge(10), a.find(7), first(j for j in 0..n if a[j] > 0), max_by(0..n, j => a[j]), none: "none")
