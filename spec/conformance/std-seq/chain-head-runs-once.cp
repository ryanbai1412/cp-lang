// expect: ok, output "1\n2\n3\n3"
// A method chain evaluates its head once, however the methods re-lower it.
var calls = 0
fn make() -> List<Int>:
    calls += 1
    return [3, 1, 2]
print(make().sorted().first!)
print(make().rev.first!)
print(sum(make().drop(1)))
print(calls)
