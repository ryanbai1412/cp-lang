// expect: ok, output "YES NO YES"
let z = 0
let n: ?Int = None
print(n.is_none_or(v => v / z > 0), Some(2).is_none_or(v => v > 3), Some(5).is_none_or(v => v > 3))
