// expect: E0606
// tool: check
// has: "`sorted` gives a new `List`"
// `sort` changes a place in place; a domain is none, and the error names the
// allocating `sorted` (decision 627).
domain I = 0..3
let w: [I]Int = [I] i => [5, 9, 1][i.ord]
let order = I.sort(by: i => w[i]) // <- here
print(order)
