// error: `a` can't be passed `mut` to `g`: `() => a.len` captures it and uses it during the same call
fn g(mut xs: List<Int>, h: () -> Int) -> Int:
    xs.push(h())
    return xs.len
var a = [1]
print(g(mut a, () => a.len))
