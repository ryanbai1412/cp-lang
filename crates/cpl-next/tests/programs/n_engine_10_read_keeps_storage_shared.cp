// expect: ok, output "1 2\n9 1\n9 5"
// Reading a cell of storage two holders share leaves it shared; the first write makes the
// writer's own copy, so the other holder never sees it.
var a = [1, 2, 3]
var b = a.clone()
print(b[0], a[1])
a[0] = 9
print(a[0], b[0])
b[0] = 5
print(a[0], b[0])
