// expect: ok, output "2 3 4"
// rule: R08.1.4 R11.3.4
// A closure or fn item meets a free fn whose `self` is a fn type, as it meets that parameter.
use std.algebra.{opt}
let f = (a: Int, b: Int) => a + b
fn add(a: Int, b: Int) -> Int = a + b
let m = f.opt()
print(m.op(None, Some(2))!, f.opt().op(Some(1), Some(2))!, add.opt().op(Some(4), None)!)
