// expect: ok, output "5\n4"
// rule: 08.1.9
// A type param named in several bounds is solved from all of them: two `[V]` arrays, or a list and a str, agree on `K` (decision 543).
domain V = 0..3
fn pair<A: Keyed<K, _>, B: Keyed<K, _>, K>(a: A, b: B) -> Int = a.len + b.len
let a: [V]Int = [1, 2, 3]
let b: [V]Int = [4, 5, 6]
print(pair(a, b) - 1)
print(pair([1, 2], "ab"))
