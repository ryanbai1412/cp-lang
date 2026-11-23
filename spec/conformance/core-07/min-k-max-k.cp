// expect: ok, output "1 2\n4 5"
// rule: 07.9.3
// `min_k`/`max_k` give the k smallest/largest in no set order.
let a = [5, 1, 4, 2]
print(a.min_k(2).sorted())
print(a.max_k(2).sorted())
