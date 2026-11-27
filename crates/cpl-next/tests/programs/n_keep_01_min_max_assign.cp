// expect: ok, output "5 3\n1 0\n2 13\n2.5000000000\n5 5"
// `x min= e` and `x max= e`: `None` is the identity on both sides, and as a condition it is
// true when the place strictly improved.
domain V = 0..3
var best: ?Int = None
best min= 5
best min= None
let a = best!
best min= 3
print(a, best!)
var d: ?Int = None
print(if d min= 5: 1 else: 0, if d min= 7: 1 else: 0)
var hi = 2
hi max= 1
var dist: [V]Int = [V].filled(9)
dist[V(1)] min= 4
dist[V(1)] max= 2
print(hi, dist[V(1)] + dist[V(2)])
var r: F64 = 1e18
r min= 2.5
print(r)
var parent = 0
if dist[V(2)] min= 5: parent = 5
if dist[V(2)] min= 6: parent = 6
print(parent, dist[V(2)])
