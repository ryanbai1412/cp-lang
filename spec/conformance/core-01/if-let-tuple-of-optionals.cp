// expect: ok, output "5\n0"
// rule: 01.7.1
// `if let` destructures a tuple pattern against the whole value, as `match` does.
let a: ?Int = Some(3)
let b: ?Int = Some(2)
if let (Some(l), Some(r)) = (a, b):
    print(l + r)
let c: ?Int = None
if let (Some(l), Some(r)) = (a, c):
    print(l + r)
else:
    print(0)
