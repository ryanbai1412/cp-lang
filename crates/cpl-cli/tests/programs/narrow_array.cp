struct Box<V>(dep: [V]0..=V.count, far: [V]Int)

fn bump_narrow<V>(mut xs: [V]0..=1_000_000, u: V, delta: Int):
    xs[u] += delta

fn bump_wide<V>(mut xs: [V]Int, u: V, delta: Int):
    xs[u] += delta

fn make_box<V>() -> Box<V>:
    var dep: [V]0..=V.count = 0
    var far: [V]Int = 0
    for v in V:
        dep[v] = v.ord
        far[v] = v.ord * 1_000_000_000
    return Box(dep, far)

let n: 1..=100 = read()
domain V = 1..=n

var a: [V]0..=1_000_000 = 0
for v in V:
    a[v] = 1_000_000 - v.ord * 100
bump_narrow<V>(mut a, V(1), -50)
a[V(2)] max= 999_999
a[V(3)] -= 25

var prod_sum = 0
for v in V:
    prod_sum += a[v] * a[v]

var s: [V]-1_000_000..=1_000_000 = 0
for v in V:
    s[v] = (v.ord - 2) * 300_000
var neg_sum = 0
for v in V:
    neg_sum += s[v] * s[v]

var sorted_a: [V]0..=1_000_000 = 0
for v in V:
    sorted_a[v] = a[v]
sorted_a.sort()

var wide_a: [V]0..=1_000_000 = 0
for v in V:
    wide_a[v] = v.ord * 10
bump_wide<V>(mut wide_a, V(1), 5)

var viewed: List<0..=1_000_000> = []
for i in 0..n:
    viewed.push((n - i) * 1000)
viewed.sort()
let vslice = viewed[1..n]
var view_sum = 0
for x in vslice:
    view_sum += x

let b = make_box<V>()
var dep_sum = 0
for v in V:
    dep_sum += b.dep[v] * b.dep[v] + b.far[v]

print(prod_sum, neg_sum, sorted_a[V(1)], sorted_a[V(n)], wide_a[V(1)], view_sum, dep_sum)
