// expect: ok, output "3 3 0"
// rule: 12.14.1 12.14.2 12.14.5
// A user fn may take an entry of the intrinsics table as its body when its signature is the entry's.
use std.math.{popcount}
fn ones(x: Int) -> Int = @intrinsic("popcount")
print(ones(11), popcount(11), ones(0))
