// expect: ok, output "54\n-40"
// input: "4 3\n"
// A function value in a field of a generic struct: three closures reach one field.
struct Tab<T>:
    a: List<T>
    var total: T
    var op: (T, T) -> T

    fn at(mut self, i: Int, x: T):
        a[i] = x
        var r = a[i]
        for j in 0..a.len:
            if j != i: r = op(r, a[j])
        total = r

let n: Int = read()
let k: Int = read()
var init: List<Int> = []
for i in 0..n: init.push(i)
var t = Tab(init, 0, if k > 0: ((x, y) => x + y + k) else: ((x, y) => x * y))
t.at(1, 40)
print(t.total)
t.op = (x, y) => x - y
t.at(0, 5)
print(t.total)
