// expect: ok, output "YES NO"
// rule: 07.5.11
domain V = 0..3
let a = [V] v => Int(v) * 2
print(yes(a == [0, 2, 4]), yes(a == [0, 2, 5]))
