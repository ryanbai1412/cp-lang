// expect: ok, output "YES\nYES\nYES YES\nYES 5\n9 8 7 5 3 3 1\n1 3 2 4 5\n1 2 3 4 5"
var a = [3, 1, 2]
a.sort()
print(a == [1, 2, 3])
var b = [(1, 'b'), (0, 'z'), (1, 'a')]
b.sort(by: p => p.0)
print(b == [(0, 'z'), (1, 'b'), (1, 'a')])
print([3, 1, 2].sorted() == [1, 2, 3], [3, 1, 2].sorted(desc: true) == [3, 2, 1])
print([4, 5].last == Some(5), [4, 5][1..2].last ?? -1)
fn g<S: IndexedMut>(mut s: S) = sort(mut s, by: x => -x)
var c = [5, 9, 1, 7, 3, 3, 8]
g(mut c)
print(c)
var d = [(2, 1), (1, 2), (2, 3), (1, 4), (0, 5)]
sort(mut d, by: p => p.0, desc: true)
print(d.map(p => p.1).collect())
let e = sorted(d, by: p => p.1)
print(e.map(p => p.1).collect())
