// expect: ok, output "YES"
use std.strings.{join, replace}
print(yes(", ".join(["a", "b", "c"]) == "a, b, c" and " ".join([1, 2, 3]) == "1 2 3" and "-".join(["x"]) == "x" and "aaaa".replace("aa", "b") == "bb" and "aaa".replace("aa", "b") == "ba" and "banana".replace("an", "") == "ba"))
