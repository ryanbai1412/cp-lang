// The methods the library adds to the types of the language need no
// `use`: `get_or_insert_with` on a map, `sign` on an `Int`. `Dsu.merge`
// says which root stayed, and `Comb.inv(i)` is 1 / i.
let n: Int = read()
domain P = 0..n
var ids = Map<Str, P>()
fn id(name: Str) -> P:
    let next = ids.len
    ids.get_or_insert_with(name, () => P(next))
print([id(read()) for _ in 0..n + 2])
var hash = Map<Int, List<Int>>()
print(hash.get_or_insert_with(3, () => [1]), hash.get_or_insert_with(3, () => [2]))
print(hash[3], 7.sign, (n - 9).sign, (n - n).sign)
domain V = 0..n
var d = Dsu<V>()
var weight = [V] v => v.idx + 1
for (a, b) in [(0, 1), (1, 2), (0, 2)]:
    if let (kept, merged) = d.merge(V(a), V(b)):
        weight[kept] += weight[merged]
        print(kept == d.find(V(a)), weight[kept])
    else: print("one set")
let c = Comb<998244353>()
print(c.inv(1), c.inv(2), c.inv(n) * Mod<998244353>(n))
