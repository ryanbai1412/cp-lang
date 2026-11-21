// A `Set` or `Map` keyed by a list, never walked in order, is a hash table: the key hashes
// by its length and its elements.
let n: Int = read()
var seen: Set<List<Int>> = Set()
var count: Map<List<Int>, Int> = Map()
var firsts: Set<List<(Int, Str)>> = Set()
var fresh = 0
for _ in 0..n:
    let k: Int = read()
    let xs: [0..k]Int = read()
    let key = sorted([x for x in xs])
    if key not in seen:
        seen.insert(key)
        fresh += 1
    count[key] = (count.get(key) ?? 0) + 1
    firsts.insert([(x, "{x}") for x in key])
print(fresh, seen.len, firsts.len)
print(count[[1, 2, 3]], count.get([9]) ?? 0, [] in seen, [2, 1] in seen)
