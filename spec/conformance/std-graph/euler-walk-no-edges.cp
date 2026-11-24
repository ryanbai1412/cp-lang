// expect: ok, output "1"
domain V = 0..3
domain E = 0..0
let es: [E](u: V, v: V) = []
print(euler_walk(es, directed: false)!.vertices.collect().len)
