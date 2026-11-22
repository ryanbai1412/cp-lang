// expect: ok, output "YES"
use std.strings.{parse_int}
print(yes("ff".parse_int(base: 16) == Some(255) and "-12".parse_int() == Some(-12) and "12a".parse_int() == None and "".parse_int() == None and "-".parse_int() == None))
