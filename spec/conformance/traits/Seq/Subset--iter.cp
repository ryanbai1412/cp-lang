// expect: ok, output "0\n2"
// engines: next
domain V = 0..4
let s = Subset<V>.single(V(0)) | Subset<V>.single(V(2))
for v in s: print(v)
