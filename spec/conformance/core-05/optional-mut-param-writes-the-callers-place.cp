// expect: ok, output "YES NO"
// rule: 05.4.5
domain V = 0..2
fn mark(v: V, seen: ?mut [V]Bool = None):
    if let s = seen:
        s[v] = true
var a: [V]Bool = false
mark(V(1), seen: mut a)
mark(V(0))
print(yes(a[V(1)]), yes(a[V(0)]))
