// expect: ok, output "YES NO"
// engines: next
domain V = 0..4
let s = Subset<V>.single(V(0)) | Subset<V>.single(V(2))
print(yes(s.contains(V(2))), yes(s.contains(V(1))))
