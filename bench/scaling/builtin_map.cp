// The built-in ordered Map and Set of n keys.
let n: Int = read()
var m: Map<Int, Int> = Map()
var st: Set<Int> = Set()
var s0 = 0
for i in 0..n:
    let k = i * 7919 % n
    m[k] = i
    if st.insert(k): s0 += 1 // timed: builtin.Set.insert
var s = 0
for i in 0..64 * n:
    for k in m.keys: // timed: builtin.Map.keys
        s += k
        break
    for v in m.values: // timed: builtin.Map.values
        s += v
        break
for i in 0..4 * n:
    let k = i * 13 % n
    s += m.get(k) ?? 0 // timed: builtin.Map.get
    if m.contains(k): s += 1 // timed: builtin.Map.contains
    s += (m.first_ge(k) ?? (0, 0)).0 // timed: builtin.Map.first_ge
    s += (m.first_gt(k) ?? (0, 0)).0 // timed: builtin.Map.first_gt
    s += (m.last_le(k) ?? (0, 0)).0 // timed: builtin.Map.last_le
    s += (m.last_lt(k) ?? (0, 0)).0 // timed: builtin.Map.last_lt
    s += (m.range(k..k + 8).first ?? (0, 0)).0 // timed: builtin.Map.range
    let v = m.remove(k)! // timed: builtin.Map.remove
    m[k] = v
    let f = m.pop_first()! // timed: builtin.Map.pop_first
    m[f.0] = f.1
    let l = m.pop_last()! // timed: builtin.Map.pop_last
    m[l.0] = l.1
    if st.contains(k): s += 1 // timed: builtin.Set.contains
    if st.remove(k): s += 1 // timed: builtin.Set.remove
    if st.add(k): s += 1 // timed: builtin.Set.add
    st.set(k, false) // timed: builtin.Set.set
    st.set(k, true) // timed: builtin.Set.set
    s += st.first_ge(k) ?? 0 // timed: builtin.Set.first_ge
    s += st.first_gt(k) ?? 0 // timed: builtin.Set.first_gt
    s += st.last_le(k) ?? 0 // timed: builtin.Set.last_le
    s += st.last_lt(k) ?? 0 // timed: builtin.Set.last_lt
    s += st.range(k..k + 8).first ?? 0 // timed: builtin.Set.range
    let p = st.pop_first()! // timed: builtin.Set.pop_first
    st.add(p)
    let q = st.pop_last()! // timed: builtin.Set.pop_last
    st.add(q)
for r in 0..16:
    var w: Set<Int> = Set()
    for i in 0..n: w.add(i)
    s += w.drain(after: n / 4, while: x => x < n / 2).len // timed: builtin.Set.drain
for i in 0..64 * n:
    s += st.min() ?? 0 // timed: builtin.Set.min
    s += st.max() ?? 0 // timed: builtin.Set.max
for r in 0..64:
    s += m.items.len // timed: builtin.Map.items
    var c = m.clone() // timed: builtin.Map.clone
    var d = st.clone() // timed: builtin.Set.clone
    s += (d.sorted().first ?? 0) // timed: builtin.Set.sorted
    c.clear() // timed: builtin.Map.clear
    d.clear() // timed: builtin.Set.clear
    s += c.len + d.len
for i in 0..64 * n:
    m.reserve(i) // timed: builtin.Map.reserve
print(s + s0 + m.len)
