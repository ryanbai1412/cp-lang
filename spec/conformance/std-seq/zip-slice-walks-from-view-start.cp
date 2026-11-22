// expect: ok, output "7 1\n8 2"
// rule: 12.4.13
let s = [5, 6, 7, 8]
for (x, y) in zip(s[2..], [1, 2, 3]):
    print(x, y)
