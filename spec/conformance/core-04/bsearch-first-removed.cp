// expect: E0503 at 3
// rule: 04.8.3
domain V = 1..=5
let a = [V] v => Int(v)
let c = a.keys.bsearch_first(i => a[i] >= 3)
