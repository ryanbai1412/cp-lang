// expect: ok, output "YES\nYES\nYES"
domain V = 0..3
var d = Dsu<V>()
print(yes(d.merge(V(1), V(0)) == Some((kept: V(1), merged: V(0)))))
var e = Dsu<V>()
e.unite(V(0), V(1))
print(yes(e.merge(V(2), V(1)) == Some((kept: V(0), merged: V(2)))))
var f = Dsu<V>()
f.unite(V(0), V(1))
print(yes(f.merge(V(1), V(0)) == None))
