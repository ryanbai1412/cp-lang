// `.len` of a `Subset<V>` is how many members it has, not how many subsets
// there are; `.is_empty` is the subset with none.
let n: 1..=10 = read()
domain V = 0..n
var ans = 0
var empties = 0
for mask in Subset<V>:
    if mask.is_empty: empties += 1
    if all(not (u in mask and V(u.ord + 1) in mask) for u in V if u.ord + 1 < n):
        ans max= mask.len
print(ans, empties)
print(Subset<V>.empty.len, Subset<V>.full.len)
