// expect: ok, output "YES YES 3 YES"
// rule: 08.2.9
// `if V: Domain` holds when `V` is a domain, so a generic body can keep an array over it, and
// takes the other branch for a plain value type.
struct Tab<V>(d: ?[V]Int, m: HashMap<V, Int>)
fn mk<V: Hash + Eq>() -> Tab<V>:
    if V: Domain:
        return Tab(d: Some([V] _ => 0), m: HashMap<V, Int>())
    Tab(d: None, m: HashMap<V, Int>())
fn ordered<V>() -> Bool:
    if V: Ord: return true
    false
domain K = 0..3
let t = mk<Int>()
let u = mk<K>()
print(yes(t.d == None), yes(u.d != None), u.d!.len, yes(ordered<K>()))
