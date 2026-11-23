// `swap` of two elements of an array of optional scalars (`[V]?Int`) and
// of plain ones takes each element where it is, on the VM too.
let n: 1..=8 = read()
domain V = 0..n
var c: [V]?Int = [V] v => if Int(v) % 3 == 0: None else: Some(Int(v) * 10)
var a: [V]Int = [V] v => Int(v) * Int(v)
let i: V = read()
let j: V = read()
if i != j:
    swap(mut c[i], mut c[j])
    swap(mut a[i], mut a[j])
print([c[v] ?? -1 for v in V])
print(a)
