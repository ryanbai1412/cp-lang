// expect: ok, output "1 1"
// rule: 07.4.2
let a = [1, 2, 3]
print(if a[1..] == [2, 3]: 1 else: 0, if a.starts_with([1, 2]): 1 else: 0)
