// expect: ok, output "YES"
use std.search.{parallel_bsearch}
let need = [2, 9]
var c = 0
print(parallel_bsearch(2, 0..4, reset: () => c = 0, apply: _ => c += 1, ok: q => c >= need[q]) == [Some(1), None])
