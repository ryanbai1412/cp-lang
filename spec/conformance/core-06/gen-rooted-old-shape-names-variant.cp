// expect: E0201
// rule: 02.7.1
// A variant written in another case names the expected enum's spelling, read from its declaration.
domain V = 0..5
let t = gen<Rooted<V>>(shape: star) // <- here
print(V.count)
