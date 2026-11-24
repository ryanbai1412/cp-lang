// expect: ok, output "5 3\n6 1\n7 2"
var xs = [3, 1, 2]
for (k, x) in xs.keyed(keys: 5..).items: print(k, x)
