// expect: ok, output "3 2 1"
let a = [3, 1, 2]
print(split_best([0..3], range: g => g, best: (_, r) => r.map(i => (a[i], i)).max()!).collect())
