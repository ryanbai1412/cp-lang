// expect: ok, output "YES NO NO\nYES NO YES"
// rule: R08.1.6
// `==` on a list and an optional is core's ordinary `Eq` impl for each, bounded by its value's.
let a = [1, 2, 3]
print(yes(a == [1, 2, 3]), yes(a == [1, 2]), yes(a == [1, 2, 4]))
let x: ?Int = Some(2)
let n: ?Int = None
print(yes(x == Some(2)), yes(x == n), yes(n == None))
