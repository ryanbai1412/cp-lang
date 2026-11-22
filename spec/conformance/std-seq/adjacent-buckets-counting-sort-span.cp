// expect: ok, output "10 20\n20 30\n0\n2 4 3 1 1\n1\n0 2\n2 4 6 5 7 9 1\n4 1 4"
print([10, 20, 30].adjacent.collect())
let e: List<Int> = []
print(e.adjacent.collect().len)
domain K = 0..2
var t = [3, 1, 2, 1, 4]
t.counting_sort(by: x => K(x % 2))
print(t)
domain V = 0..3
var a: [V]K = [K(1), K(0), K(1)]
print(a.buckets())
var u = [5, 2, 7, 4, 9, 6, 1]
u.counting_sort(by: x => K(x % 2))
print(u)
domain W = 0..6
let r = span(W(1).l, W(4).r)
print(r.len, r.first!, r.last!)
