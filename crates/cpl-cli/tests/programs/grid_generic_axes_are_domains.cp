// `[R, C]T` given an anonymous grid: each axis is a domain the grid sizes.
fn cells<R, C, T>(a: [R, C]T) -> Int = R.count * C.count
fn total<R, C>(a: [R, C]Int) -> Int:
    var s = 0
    for r in R:
        for c in C:
            s += a[r, c]
    s
fn len<D, T>(a: [D]T) -> Int = D.count
print(len([1, 2, 3]), cells([2, 3] (r, c) => r + c))
let g = [3, 4] (r, c) => r * 10 + c
print(cells(g), total(g), total([1, 2] (r, c) => r + c))
