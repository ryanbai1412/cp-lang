// expect: ok, output "0 9 1 2 5\n9 4\n5 1 2 0\n3 4 | 7 | 1\n2 1 0\nNO"
// rule: R02.6.6
// List's insert, remove, swap, extend and reserve are plain CP over core's `raw_take`, which
// moves a cell out of storage, and `raw_cap`, storage of unset cells.
var a = [0, 1, 2]
a.reserve(10)
a.insert(1, 9)
a.push(5)
print(a)
let r = a.remove(1)
print(r, a.len)
a.swap(0, 3)
print(a)
var b: List<List<Int>> = [[1], []]
b.insert(0, [3, 4])
b.extend([[7]])
b.swap(1, 3)
let c = b.remove(0)
print(c, "|", b[0], "|", b[2])
print(c.len, b[2].len, b[1].len)
var bs = [true]
bs.insert(0, false)
bs[1] = false
print(yes(bs[0] or bs[1]))
