// CSES 1158 Book Shop: max pages with total price at most x (0/1 knapsack).
let (n, x): (1..=1000, 1..=1e5) = read()
let price: [0..n]1..=1e5 = read()
let pages: [0..n]1..=1000 = read()
var best: [0..=x]Int = 0
for i in 0..n:
    for c in (price[i]..=x).rev:
        best[c] max= best[c - price[i]] + pages[i]
print(best[x])
