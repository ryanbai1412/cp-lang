fn bump_all<D>(mut a: [D]Int, by: Int):
    for i in D: a[i] += by
    a[0] += 100

fn ends<D>(a: [D]Int) -> (Int, Int) = (a[D.first], a[D.last])

struct Acc:
    cells: [0..4]Int

let n: Int = read()
var a: [0..n]Int = [0..n] i => i
bump_all(mut a, 5)
print(a, ends(a))
var s = Acc([1, 2, 3, 4])
bump_all(mut s.cells, 1)
print(s.cells, ends(s.cells))
var rows = [[0..2] i => i, [0..2] i => 10 * i]
bump_all(mut rows[1], 2)
print(rows[0], rows[1])
var b: [1..=n]Int = [1..=n] i => i
bump_all(mut b, 1)
