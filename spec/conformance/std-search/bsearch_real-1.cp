// expect: ok, output "YES"
use std.search.{bsearch_real}
print(yes(bsearch_real(0.0, 8.0, x => x >= 3.0, steps: 3) == Some(3.0)))
