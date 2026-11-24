// expect: R0401 (checked)
use std.math.{mod_kth_root}
print(mod_kth_root(0, 2, 7) ?? -1)
print(mod_kth_root(2, 0, 7) ?? -1) // <- here
