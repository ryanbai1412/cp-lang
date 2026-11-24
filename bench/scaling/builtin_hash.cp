// The built-in HashMap and HashSet of n keys.
let n: Int = read()
var h: HashMap<Int, Int> = HashMap()
var hs: HashSet<Int> = HashSet()
for i in 0..n:
    h[i * 7919 % n] = i
    hs.add(i * 7919 % n)
var s = 0
for r in 0..4:
    for i in 0..n:
        let k = i * 7919 % n
        hs.remove(k)
        if hs.insert(k): s += 1 // timed: builtin.HashSet.insert
for i in 0..64 * n:
    for k in h.keys: // timed: builtin.HashMap.keys
        s += k
        break
    for v in h.values: // timed: builtin.HashMap.values
        s += v
        break
    s += h.len // timed: builtin.HashMap.len
    s += hs.len // timed: builtin.HashSet.len
    if h.is_empty: s += 1 // timed: builtin.HashMap.is_empty
    if hs.is_empty: s += 1 // timed: builtin.HashSet.is_empty
for i in 0..4 * n:
    let k = i * 13 % n
    s += h.get(k) ?? 0 // timed: builtin.HashMap.get
    if h.contains(k): s += 1 // timed: builtin.HashMap.contains
    let w = h.remove(k)! // timed: builtin.HashMap.remove
    h[k] = w
    if hs.contains(k): s += 1 // timed: builtin.HashSet.contains
    if hs.remove(k): s += 1 // timed: builtin.HashSet.remove
    if hs.add(k): s += 1 // timed: builtin.HashSet.add
    hs.set(k, false) // timed: builtin.HashSet.set
    hs.set(k, true) // timed: builtin.HashSet.set
for r in 0..64:
    s += hs.min() ?? 0 // timed: builtin.HashSet.min
    s += hs.max() ?? 0 // timed: builtin.HashSet.max
    s += h.items.len // timed: builtin.HashMap.items
for r in 0..64:
    var c = h.clone() // timed: builtin.HashMap.clone
    var d = hs.clone() // timed: builtin.HashSet.clone
    s += (d.sorted().first ?? 0) // timed: builtin.HashSet.sorted
    c.clear() // timed: builtin.HashMap.clear
    d.clear() // timed: builtin.HashSet.clear
    s += c.len + d.len
for r in 0..1024:
    var e: HashMap<Int, Int> = HashMap()
    e.reserve(n) // timed: builtin.HashMap.reserve
    s += e.len
print(s)
