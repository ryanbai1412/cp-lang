// error: `grow()` changes `xs` while `xs` is lent to this call
var xs: List<Int> = [1, 2, 3]
fn grow() -> Int:
    xs.push(4)
    return 0
fn total(ys: List<Int>, k: Int) -> Int:
    return sum(ys) + k
print(total(xs, grow()))
