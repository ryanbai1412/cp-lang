// expect: ok, output "35 20"
// input: "5\n"
trait Monoid:
    fn id(self) -> Int
    fn op(self, a: Int, b: Int) -> Int

struct Sum
impl Monoid for Sum:
    fn id(self) -> Int: 0
    fn op(self, a: Int, b: Int) -> Int: a + b

fn fold<M: Monoid>(xs: List<Int>, m: M) -> Int:
    var acc = m.id()
    for i in 0..xs.len: acc = m.op(acc, xs[i])
    acc

fn apply(f: Int -> Int, x: Int) -> Int: f(x)

let n: Int = read()
var xs: List<Int> = List.filled(0, 0)
for i in 0..n: xs.push(i * i)
xs[0] += 5
var f = (x: Int) => x + 1
if n > 3: f = x => x * 2
print("{fold(xs, Sum())} {apply(f, 10)}")
