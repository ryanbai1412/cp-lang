// expect: ok, output "0 3\n1 1\n2 4\n16"
// rule: 06.1.1
@get fn ks<T>(self: List<T>) -> Range<Int> = 0..self.len
fn total<T>(self: List<T>, f: T -> Int) -> Int:
    var s = 0
    for x in self:
        s += f(x)
    s
let a = [3, 1, 4]
for i in a.ks:
    print(i, a[i])
print(a.total(x => x * 2))
