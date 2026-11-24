// expect: ok, output "2 YES\n9\n4 6"
// input: "5 7 0 9\n4 6\n"
// rule: 08.1.2
// `read_all` is a std body over `read<?T>()`: with `until` it stops at the sentinel and leaves it out, reading goes on after it, and the plain form reads to end of input.
use std.io.{read_all}
let head = read_all<Int>(until: x => x == 0)
print(head.len, yes(head == [5, 7]))
print(read<Int>())
let rest = read_all<Int>()
print(rest[0], rest[1])
