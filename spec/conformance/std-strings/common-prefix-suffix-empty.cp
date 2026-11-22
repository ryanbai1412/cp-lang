// expect: ok, output "0 3 0"
use std.strings.{common_prefix_len, common_suffix_len}
print("".common_prefix_len("ab"), "abc".common_prefix_len("abc"), "ab".common_suffix_len("ba"))
