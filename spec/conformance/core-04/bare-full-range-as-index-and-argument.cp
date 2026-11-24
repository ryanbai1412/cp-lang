// expect: ok, output "3 2 2\n3"
// rule: 04.8.6
let a = [1, 2, 3]
print(a[..].len, a[1..].len, a[..2].len)
print((3..).find(x => x % 3 == 0)!)
