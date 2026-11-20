// min / max of a slice are ?Int: a slice keeps its parent index type but may be empty
let n: 1..=5 = read()
domain V = 1..=n
var a: [V]Int = read()
let f = V.first
print(min(a[f..f]), max(a[f..f]), sum(a[f..f]))
print(min(a[f..=f]), max(a[f..=V.last]), sum(a[f..=V.last]))
if let m = min(a[f..f]): print(m)
else: print(0)
