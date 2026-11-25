// expect: ok, output "51\n240\n8\n16"
// input: "6 3\n"
fn map(xs: List<Int>, f: Int -> Int) -> List<Int>:
    var out: List<Int> = []
    for i in 0..xs.len: out.push(f(xs[i]))
    out
fn fold(xs: List<Int>, init: Int, f: (Int, Int) -> Int) -> Int:
    var acc = init
    for i in 0..xs.len: acc = f(acc, xs[i])
    acc
fn compose(f: Int -> Int, g: Int -> Int) -> Int -> Int = x => g(f(x))
let n: Int = read()
let k: Int = read()
var xs: List<Int> = []
for i in 0..n: xs.push(i)
let ys = map(xs, x => x * k)
let zs = map(ys, x => x + 1)
print(fold(zs, 0, (a, b) => a + b))
print(fold(zs, 1, (a, b) => (a * b) % 1000))
let h = compose(x => x + k, x => x * 2)
print(h(1))
print(map(xs, h)[n - 1])
