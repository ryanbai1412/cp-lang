// `swap` of two elements of a packed array (`[V]Int`, `[V]Bool`) moves each
// element through a copy, as a store into one does.
let n: 1..=8 = read()
domain V = 0..n
var a: [V]Int = [V] v => Int(v)
var b: [V]Bool = [V] v => Int(v) % 2 == 0
let i: V = read()
let j: V = read()
if i != j:
    swap(mut a[i], mut a[j])
    swap(mut b[i], mut b[j])
print(a)
print([if b[v]: 1 else: 0 for v in V])
