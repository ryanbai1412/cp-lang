// expect: E0606
// rule: 12.14.6
// A struct with a fn field derives no `Hash`, so `HashMap<K: Hash, V>` rejects it as a key at the type.
struct F(f: Int -> Int)
var m: HashMap<F, Int> = HashMap()  // <- here
print(m.len)
