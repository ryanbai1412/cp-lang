// expect: ok, output "YES"
domain V = 0..3
let d = Dsu<V>()
print(d.components == 3)
