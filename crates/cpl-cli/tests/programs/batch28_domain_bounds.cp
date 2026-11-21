domain V = 1..=6
var a: [V]Int = [V] v => v.label * 10
var b: [V]Int = 25
let (v1, v2) = (V(2), V(4))
print(a[v1..v2])
print(a[v1..=v2])
a[V(2)..V(4)] = 0
print(a)
a[v1..=v2] = 7
print(a)
a[v1..=v2] min= b[v1..=v2]
print(a)
a[v2..] max= b[v2..]
print(a)
struct Jump:
    jdist: List<Int>

    fn far(self, p: Int) -> Int = jdist[p] + 1

domain P = 0..3
var ids = Map<Str, P>()
for name in ["a", "b", "a", "c"]: print(ids.get_or_insert_with(name, () => P(ids.len)))
print(Jump([1, 2]).far(1))
