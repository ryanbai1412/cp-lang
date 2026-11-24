// expect: E0201
// rule: 07.4.16
// gap: stdlib 2026-11-24T05Z
// std has no `suffix_fold` (decision 511): a suffix fold is `a.rev.scan(op)`.
use std.prefix.{suffix_fold} // <- here
let a = [2, 3, 4]
print(suffix_fold(*, a)[1])
