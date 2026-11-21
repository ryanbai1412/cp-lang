var xs = [3, 1, 2]
var calls = 0

fn grow(x: Int) -> Int:
    xs.push(x * 10)
    -x

fn count(x: Int) -> Int:
    calls += 1
    x

let ys = xs.sorted(by: grow)
print(xs)
print(ys)
xs.sort(by: count)
print(xs, calls)
