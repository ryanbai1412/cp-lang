// expect: ok, output "5"
domain V = 0..4
print(Subset<V>.single(V(0)) | Subset<V>.single(V(2)))
