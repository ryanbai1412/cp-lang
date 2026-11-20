// Fns named like ones in `std`, called with lists: C++ must pick these.
fn apply(a: List<Int>, b: List<Int>) -> List<Int> = [a[i] + b[i] for i in 0..a.len]
fn swap(a: List<Int>, b: List<Int>) -> Int = a.len - b.len
fn distance(a: List<Int>, b: List<Int>) -> Int = sum(abs(a[i] - b[i]) for i in 0..a.len)
fn find(a: List<Int>, x: Int) -> Int = a.len + x
fn count(a: List<Int>, b: List<Int>) -> Int = a.len * b.len
fn get(a: (Int, Int)) -> Int = a.0 + a.1
let xs = [1, 2, 3]
let ys = [10, 20, 30]
print(apply(xs, ys), swap(xs, ys), distance(xs, ys), find(xs, 4), count(xs, ys), get((1, 2)))
