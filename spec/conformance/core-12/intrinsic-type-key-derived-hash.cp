// expect: ok, output "2 1"
// rule: 12.14.6
// A struct whose fields are all `Hash` derives it and meets `HashSet<K: Hash>` and `HashMap<K: Hash, V>`.
struct P(x: Int, y: Str)
let s: HashSet<P> = HashSet([P(1, "a"), P(2, "a"), P(1, "a")])
var m: HashMap<P, Int> = HashMap()
m[P(1, "a")] = 1
m[P(1, "a")] += 1
print(s.len, m[P(1, "a")] - 1)
