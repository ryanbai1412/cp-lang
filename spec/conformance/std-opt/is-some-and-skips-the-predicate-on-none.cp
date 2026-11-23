// expect: ok, output "YES NO NO"
let z = 0
let n: ?Int = None
print(Some(4).is_some_and(v => v > 3), Some(2).is_some_and(v => v > 3), n.is_some_and(v => v / z > 0))
