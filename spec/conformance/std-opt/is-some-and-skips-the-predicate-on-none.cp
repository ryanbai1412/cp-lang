// expect: ok, output "YES NO NO"
let z = 0
let n: ?Int = None
print(yes(Some(4).is_some_and(v => v > 3)), yes(Some(2).is_some_and(v => v > 3)), yes(n.is_some_and(v => v / z > 0)))
