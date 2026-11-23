// expect: ok, output "3 1\n2 9\n1 5\nYES NO\n3 1"
use std.derive.{lt_fields, eq_fields, show_fields}
struct Job(day: Int, cost: Int)
impl Ord for Job: fn lt(a, b) = lt_fields(b, a)
var js = [Job(1, 5), Job(3, 1), Job(2, 9)]
js.sort()
for j in js: print(j.day, j.cost)
print(yes(eq_fields(Job(1, 5), Job(1, 5))), yes(eq_fields(Job(1, 5), Job(5, 1))))
print(show_fields(js[0]))
