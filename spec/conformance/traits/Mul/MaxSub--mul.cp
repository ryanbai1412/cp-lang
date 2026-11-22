// expect: ok, output "-6 0"
use std.algebra.{MaxSub}
print((MaxSub.of(-3) * 2).try_best!, (MaxSub.of(-3) * 0).try_best!)
