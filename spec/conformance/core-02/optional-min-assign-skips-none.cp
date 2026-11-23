// expect: ok, output "5"
// `?T min= ?T` keeps the smaller present value; `None` on either side is the identity (round 26, H03).
var a: ?Int = None
let b: ?Int = 5
let c: ?Int = None
a min= b
a min= c
print(a ?? -1)
