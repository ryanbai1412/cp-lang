// expect: ok, output "3 5 4 4"
// rule: 07.4.11 07.7.3
let a = [1, 2, 3, 4, 5]
print(a.drop(2)[2], a.take(10).len, a.drop_last(1).len, a.take_last(2).first!)
