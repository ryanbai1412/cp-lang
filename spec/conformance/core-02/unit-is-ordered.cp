// expect: ok, output "YES NO YES\nYES"
// `()` has one value, so it is equal to itself and ordered like the empty tuple; a generic
// `T: Ord` compares and folds it on both engines.
fn least<T: Ord>(xs: List<T>) -> T:
    var at = 0
    for i in 0..xs.len:
        if xs[i] < xs[at]: at = i
    xs[at]
print(yes(() == ()), yes(() < ()), yes(() <= ()))
print(yes(least([(), ()]) == min((), ())))
