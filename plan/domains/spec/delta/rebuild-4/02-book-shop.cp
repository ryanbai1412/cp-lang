// CSES 1158 Book Shop
// in: n x, then n prices, then n page counts; out: the most pages buyable with total price <= x
let (n, x) = read<(Int, Int)>()
domain Book = 0..n
let price = read<[Book]Int>()
let pages = read<[Book]Int>()
var best: [0..=x]Int = 0
for b in Book:
    for c in (price[b]..=x).rev:
        best[c] max= best[c - price[b]] + pages[b]
print(best[x])
