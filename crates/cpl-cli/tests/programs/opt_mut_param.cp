// A `?mut T` param: left out, the fn runs without it; given `mut x`, `if let s = seen:` writes reach `x`.
domain V = 0..4
fn mark(from: V, seen: ?mut [V]Bool = None) -> Int:
    var n = 0
    for v in V:
        if v.ord >= from.ord:
            n += 1
            if let s = seen:
                s[v] = true
    return n
fn bit(x: Bool) -> Int = if x: 1 else: 0
var seen: [V]Bool = false
print(mark(V(2)), bit(seen[V(3)]))
print(mark(V(1), seen: mut seen), bit(seen[V(0)]), bit(seen[V(3)]))
