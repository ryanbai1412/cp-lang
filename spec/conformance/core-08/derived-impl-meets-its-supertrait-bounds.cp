// expect: ok, output "2 1"
// rule: 08.1.6
// engines: next
// A derived `Ord` compares leading fields with `==`, so it asks the struct's parameters for what
// its fields' `Eq` impls ask as well as their `Ord`: `HashMap<K, Int>`'s `Eq` asks `K: Hash`,
// and the struct states it. Interpreter and C++ agree.
struct Interner<K: Hash + Eq>:
    keys: List<K>
    to_id: HashMap<K, Int>
var m = HashMap<Int, Int>()
m[7] = 0
m[9] = 1
let i = Interner([7, 9], m)
print(i.keys.len, i.to_id[9])
