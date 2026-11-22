// expect: ok, output "YES"
use std.search.{bsearch_real}
print(yes(bsearch_real(0.0, 1.0, x => x > 2.0) == None))
