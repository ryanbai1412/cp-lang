// `var` on a parameter: the fn has its own copy, which it may assign.
fn grow(var b: Int, i: Int) -> Int:
    while b < i: b *= 2
    b
fn tail(var xs: List<Int>) -> List<Int>:
    xs.push(0)
    xs
let k = 3
let ys = [1]
print(grow(k, 20), k, tail(ys), ys)
