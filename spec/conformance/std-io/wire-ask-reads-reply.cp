// expect: ok, output "? 1 2\n! 7"
// input: "7\n"
use std.io.{Wire}
let wire = Wire()
let x: Int = wire.ask("? 1 2")
print("!", x)
