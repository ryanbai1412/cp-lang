// expect: E0302
// rule: 08.1.9
// A type param named in several bounds is one type: `[V]` and `[E]` give `K` two keys (decision 543).
domain V = 0..3
domain E = 0..2
fn pair<A: Keyed<K, _>, B: Keyed<K, _>, K>(a: A, b: B) -> Int = a.len + b.len
let a: [V]Int = [1, 2, 3]
let b: [E]Int = [4, 5]
print(pair(a, b)) // <- here
