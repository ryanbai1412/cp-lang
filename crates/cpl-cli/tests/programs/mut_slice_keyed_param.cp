// A `mut` slice of a `[V]T` passed to a `mut [V]T` param keeps its keys:
// the fn reads and writes it at `V` keys, and changes the caller's array.
domain V = 1..=6
fn paint(mut x: [V]Int, by: Int):
    for k in x.keys: x[k] += by * k.ord
fn bump_first(mut x: [V]Int) -> V:
    let k = x.keys.first!
    x[k] = x[k] * 10
    k
fn inner(mut x: [V]Int):
    let c = x.keys.first!.next(1)!.l
    paint(mut x[c..], 100)
var a = [V] v => v.ord
let c = V(3).l
paint(mut a[..c], 1)
paint(mut a[c..], 2)
print(a, sep: " ")
print(bump_first(mut a[c..]))
inner(mut a[V(2).l..])
print(a, sep: " ")
fn show(mut x: [V]Int):
    print(List(x.items), sep: " ")
    x[x.keys.last!] = 0
show(mut a[V(4).l..])
print(a, sep: " ")
