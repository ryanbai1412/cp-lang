let n: 1..=5 = read()
domain V = 1..=n
var a: [V]Int = read()
let k: Int = read()
print(a[V.first!.next!])
print(a[V.at(V.first!.ord + k)!])
