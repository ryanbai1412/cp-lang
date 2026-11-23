// expect: ok, output "7 1 0 0 3 1 0\n0 1 0 1 2 2 3\n1 2 1 2\n2 YES"
// rule: 08.1.1
use std.strings.{max_rotation}
print(z_function("aabxaab"))
print(prefix_function("aabaaab"))
print("abab".prefix_matches("ab"))
print(max_rotation("baca")!, yes(max_rotation("") == None))
