// expect: ok, output "YES YES NO\n2"
// `is_some` and `is_none` are getters in std (`@get`), read like fields as its examples show.
let a: ?Int = Some(4)
let n: ?Int = None
print(yes(a.is_some), yes(n.is_none), yes(n.is_some))
var k = 0
for x in [Some(1), None, Some(3)]:
    if x.is_some: k += 1
print(k)
