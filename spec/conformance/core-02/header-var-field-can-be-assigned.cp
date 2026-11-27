// expect: ok, output "3 7"
// rule: 02.6.1
// A header field declared `var` can be assigned after construction, like a body one.
struct Acc(var total: Int, pub var n: Int):
    fn add(mut self, x: Int):
        total += x
        n += 1

var a = Acc(0, 0)
a.add(3)
a.n = 7
print(a.total, a.n)
