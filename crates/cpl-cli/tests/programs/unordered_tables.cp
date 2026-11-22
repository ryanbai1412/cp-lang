// A `Map` or `Set` that is only stored into and looked up is a hash table in the C++;
// one that is walked in order (`names`) stays a tree. Both back ends print the same.
let n: Int = read()
var cnt = Map<Int, Int>()
var seen = Set<Int>()
var names = Map<Str, Int>()
for i in 0..n:
    let x = i * 7919 % 1000
    cnt[x] = (cnt.get(x) ?? 0) + 1
    seen.insert(x % 10)
    names["k{x % 3}"] = i
print(cnt.len, yes(5 in seen), cnt[0], seen.len)
for (k, v) in names: print(k, v)

// `if k in m:` followed by `m[k]` is one lookup in the C++.
fn seen_before(x: Int) -> Int:
    if x in cnt: return cnt[x] + 1
    0
print(seen_before(7), seen_before(-1))
if "k1" in names: print(names["k1"])
