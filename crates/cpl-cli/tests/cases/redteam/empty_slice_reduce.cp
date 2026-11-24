// min / max of a slice are ?Int: a slice keeps its parent index type but may be empty
let n: 1..=5 = read()
domain V = 1..=n
var a: [V]Int = read()
let f = V.first!
print(a[f..f].min() ?? -1, a[f..f].max() ?? -1, a[f..f].sum())
print(a[f..=f].min()!, a[f..=V.last!].max()!, a[f..=V.last!].sum())
if let m = a[f..f].min(): print(m)
else: print(0)
