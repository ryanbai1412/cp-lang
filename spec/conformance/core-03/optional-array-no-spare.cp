// expect: ok, output "18446744073709551615 1"
// rule: 03.2.7
domain V = 0..3
var a: [V]?U64 = [V].filled(None)
a[V(1)] = 18446744073709551615
print(a[V(1)]!, if a[V(0)] is None: 1 else: 0)
