// A `mut` arg that is the field of a one-Int struct (packed in the VM),
// passed to a fn and to a local fn, is copied out and stored back.
struct Box:
    total: Int
var rows: List<Int> = [1, 2, 3]
var b = Box(total: 0)
fn bump(mut x: Int, mut t: Int, by: Int):
    x += by
    t += x
for i in 0..3:
    bump(mut rows[i], mut b.total, 10)
print(rows, b.total)
var c = Box(total: 0)
for i in 0..3:
    fn bump(mut t: Int, by: Int):
        t += by + i
    bump(mut c.total, 10)
print(c.total)
