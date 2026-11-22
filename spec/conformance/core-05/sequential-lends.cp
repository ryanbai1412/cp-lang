// expect: ok, output "6\n6"
var xs = [3, 1, 2]
print(xs.map(x => x).sum())
xs.sort()
print(xs.sum())
