// expect: ok, output "5 YES"
domain V = 0..5
let p = gen<Perm<V>>()
print(p.len, yes(p.sorted() == V.collect()))
