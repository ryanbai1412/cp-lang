// A closure passed beside a `mut` argument reads the lent place, and the
// callee's push lands in the caller's list.
fn use_with_cb(mut xs: List<Int>, f: () -> Int):
    xs.push(f())

var acc = 0
fn bump(d: Int):
    acc += d

for i in 1..=5:
    bump(i)

var ys: List<Int> = [10, 20]
use_with_cb(mut ys, () => ys.len)
print(acc, ys.len, ys[2])
