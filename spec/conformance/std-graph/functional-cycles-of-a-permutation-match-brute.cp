// expect: ok, output "YES"
// On permutations with many short cycles and on a single long one, cycles(), cycle_len and
// after agree with walking `p` by hand, and every vertex lies in exactly one listed cycle.
var ok = true
for trial in 0..40:
    domain V = 0..50
    var p: [V]V = [V] v => v
    let swaps = if trial % 2 == 0: rand(0, 10) else: 200
    for _ in 0..swaps:
        let (i, j) = (V(rand(0, 49)), V(rand(0, 49)))
        let t = p[i]
        p[i] = p[j]
        p[j] = t
    let f = Functional(p)
    var seen: [V]Int = [V] _ => 0
    for c in f.cycles():
        for t in 0..c.len:
            seen[c[t]] += 1
            if p[c[t]] != c[(t + 1) % c.len] or f.cycle_len(c[t]) != c.len: ok = false
            if f.after(c[t], 3 * c.len + 1) != c[(t + 1) % c.len]: ok = false
            if not f.on_cycle(c[t]) or f.root_of(c[t]) != c[t]: ok = false
        if c[0] != c.min()!: ok = false
    if any(seen[v] != 1 for v in V): ok = false
print(if ok: "YES" else: "NO")
