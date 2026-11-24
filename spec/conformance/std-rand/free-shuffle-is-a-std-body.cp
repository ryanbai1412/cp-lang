// expect: ok, output "1 2 3 4 5 | 5 3 2 4 1 | 3 2 1"
// Free `shuffle` is `std.rand`'s Fisher-Yates over the program's `rand` stream, so a run's seed
// fixes the order in `cpl run` and the built C++ alike, and a user copy of the body is the same fn.
fn mine<S: IndexedMut>(mut a: S):
    for i in (1..a.len).rev: a.swap_idx(i, rand(0, i))
var a = [1, 2, 3, 4, 5]
shuffle(mut a)
var b = [1, 2, 3]
mine(mut b)
print(a.sorted(), "|", a, "|", b)
