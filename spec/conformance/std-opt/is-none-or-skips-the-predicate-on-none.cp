// expect: ok, output "YES NO YES"
let z = 0
let n: ?Int = None
print(yes(n.is_none_or(v => v / z > 0)), yes(Some(2).is_none_or(v => v > 3)), yes(Some(5).is_none_or(v => v > 3)))
