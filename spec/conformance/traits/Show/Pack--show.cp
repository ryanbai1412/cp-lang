// expect: ok, output "1 a 2"
use std.show.{Pack}
print(Pack(1, Pack("a", ())), "{Pack(2, ())}")
