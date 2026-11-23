// expect: ok, output "YES 0 3"
domain V = 0..2
var d: [V, V]?Int = None
d[V(0), V(1)] = Some(3)
print(yes(floyd_warshall(mut d)), d[V(0), V(0)]!, d[V(0), V(1)]!)
