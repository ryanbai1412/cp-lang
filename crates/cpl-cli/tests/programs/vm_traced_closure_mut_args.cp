// A traced call of a fn value with `mut` arguments: each place is lent for
// the call, an element of a list and a field included, and put back after.
struct Box:
    total: Int
var rows: List<Int> = [1, 2, 3]
var b = Box(total: 0)
for i in 0..3:
    let k = i + 1
    fn bump(mut x: Int, mut t: Int, by: Int):
        x += by * k
        t += x
    bump(mut rows[i], mut b.total, 10)
print(rows, b.total)
