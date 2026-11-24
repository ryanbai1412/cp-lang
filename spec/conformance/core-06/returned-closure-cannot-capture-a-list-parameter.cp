// expect: E0603
// rule: 05.10.1
// A heap parameter is still captured by reference (decision 471), so a closure over it is lent and can't be returned.
fn mk(xs: List<Int>) -> (Int -> Int):
    (k: Int) => xs.len + k // <- here
print(mk([1, 2])(1))
