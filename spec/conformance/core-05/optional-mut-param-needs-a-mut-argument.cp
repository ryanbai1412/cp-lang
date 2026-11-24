// expect: E0402 at 6
// rule: 05.4.5
domain V = 0..2
fn mark(v: V, seen: ?mut [V]Bool = None):
    if let s = seen:
        s[v] = true
var a: [V]Bool = false
mark(V(1), seen: a)
