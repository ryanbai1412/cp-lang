// expect: ok, output "3 2\n6\n0\n1\n2"
// rule: R08.1.2
use std.seq.{Indexed}
fn width<S: Indexed>(s: S) -> Int = s.keys.len
fn second<S: Indexed<Int, T>, T>(s: S) -> T = s.nth(1)
print(width([4, 5, 6]), width("ab"))
print(second([4, 6]))
for k in [3, 1, 4].map(x => x * 2).keys: print(k)
