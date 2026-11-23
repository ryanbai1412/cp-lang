// expect: ok, output "0 2"
domain V = 0..4
print(Subset<V>.single(V(0)) | Subset<V>.single(V(2)))
