// expect: E0902
use std.seq.{perm_pow}
print(perm_pow([1, 2, 0], -1)[0]) // <- here
