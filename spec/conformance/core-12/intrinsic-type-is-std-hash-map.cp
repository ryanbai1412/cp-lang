// expect: ok, output "4 2 YES"
// rule: 12.14.6
// std declares `HashMap<K: Hash, V>` and `HashSet<K: Hash>` with storage from the table; both engines run it.
var m: HashMap<Int, Int> = HashMap()
m[3] = 4
let s: HashSet<Str> = HashSet(["a", "b", "a"])
print(m[3], s.len, yes(s.contains("b")))
