// expect: ok, output "YES"
use std.search.{parallel_bsearch}
print(parallel_bsearch(1, 0..0, reset: () => (), apply: _ => (), ok: _ => true) == [None])
