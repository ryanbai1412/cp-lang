// expect: ok, output "0 0"
use std.search.{split_best}
let groups: List<Range<Int>> = []
print(split_best(groups, range: g => g, best: (_, r) => (0, r.start)).count(), split_best([0..0], range: g => g, best: (_, r) => (0, r.start)).count())
