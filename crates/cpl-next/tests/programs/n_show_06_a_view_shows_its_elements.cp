// expect: ok, output "3 2 1\n4 8\n1 4 9\n7 8\n2 1\n[3 2]"
// rule: R10.3.1
// A view is `Show` by an ordinary bounded impl over its walk, and a view lent into `print` keeps the lent form.
let xs = [1, 2, 3]
print(xs.rev)
print([1, 2, 3, 4].filter(x => x % 2 == 0).map(x => x * 2))
print((1..4).map(x => x * x))
let t = [5, 6, 7, 8][2..].rebase
print(t)
print(xs[0..2].rev)
print("[{xs[1..].rev}]")
