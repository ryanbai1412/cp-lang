// expect: ok, output "5 YES"
domain V = 0..5
let g = gen<Graph<V, (to: V, w: 1..=9)>>(simple: true, connected: true)
print(V.count, yes(V.all(v => g.out(v).all(e => e.to != v and 1 <= e.w <= 9))))
