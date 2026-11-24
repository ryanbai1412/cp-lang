// expect: ok, output "5 YES"
domain V = 0..5
let t = gen<Rooted<V>>(shape: Star)
print(V.count, yes(V.all(v => t.depth(v) <= 1)))
