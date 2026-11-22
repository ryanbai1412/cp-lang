// expect: ok, output "YES"
use std.strings.{to_lower, to_upper, trim, trim_start, trim_end}
print("AbC1".to_lower() == "abc1" and "AbC1".to_upper() == "ABC1" and "  a b \n".trim() == "a b" and " \t ".trim() == "" and "  a b \n".trim_start() == "a b \n" and "  a b \n".trim_end() == "  a b")
