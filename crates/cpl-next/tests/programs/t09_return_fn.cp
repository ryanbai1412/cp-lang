// expect: ok, output "12\n40\n-4\n40"
// input: "7\n"
// A fn that returns a function value: the caller's set is "whatever pick returns".
fn inc(x: Int) -> Int = x + 1
fn dbl(x: Int) -> Int = x * 2
fn pick(b: Bool, k: Int) -> Int -> Int:
    if b: return inc
    if k > 5: return dbl
    x => x + k
fn twice(f: Int -> Int, x: Int) -> Int = f(f(x))
let k: Int = read()
var f = pick(k > 2, k)
print(twice(f, 10))
f = pick(false, k)
print(twice(f, 10))
f = x => x - k
print(twice(f, 10))
print(twice(dbl, 10))
