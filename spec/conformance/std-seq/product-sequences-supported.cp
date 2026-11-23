// expect: ok, output "4 1\n2\n1 4\n-1 3 1\n8 7\n1 -1\n5 14\n5\n1 1\n2 2 2\n1\n3 2 1 3 2 1\n1 2 0\n3\n6\n9\n2 3\n1 2 4 5\n2 6\n3 1\n40\n3 3\n1 1 2 2\n4"
// The supported use cases of handbook/product/std/sequences-usecases.md, one
// line each, in ID order: U1 U2 U3 U4 U5 U6 U7 U8 U10 U12 U13 U14 U16 U17 U18
// U19 U20 U21 U22 U23.
domain I = 0..5
let a: [I]Int = [3, 1, 4, 1, 5]
let l = [3, 1, 4, 1, 5]
print(l.find(x => x > 3)!, l.rfind(x => x < 3)!)
print(a.keys.find(i => a[i] > 3)!)
print([3, 5, 4, 5, 1].argmax()!, [3, 5, 4, 5, 1].argmin()!)
let e: List<Int> = []
print(e.max() ?? -1, [3, 1].max()!, [3, 1].max(by: x => -x)!)
print((0..100).first_true(x => x * x >= 50)!, (0..100).last_true(x => x * x < 50)!)
print([1, 3, 5].first_ge(3)!, [1, 3, 5].first_ge(6) ?? -1)
let p = l.scan(+)
print(p.fold(1..3), p.all)
print(l[1..].scan(+).fold(1..3))
for r in [1, 1, 2, 2, 2].runs():
    print(r)
print(1)
var s = [3, 1, 2]
s.sort(by: x => -x)
print(s, [3, 1, 2].sorted(desc: true))
let w = [30, 10, 20]
print(w.keys.sorted(by: i => w[i]))
for v in [1, 2, 4, 5].windows(2):
    print(v.sum())
for (k, x) in [1, 2, 3].rev.items:
    print(k, x)
    break
var g = [1, 2]
g.extend([3, 4, 5][1..])
print(g)
print([1, 2, 3].count(x => x > 1), [1, 2, 3].sum())
print([3, 3, 1, 1].sorted(desc: true).distinct().collect())
print([1, 4, 9].first_some(x => if x > 3: Some(x * 10) else: None)!)
var st = [1, 2]
st.push(3)
print(st.last!, st.pop()!)
print([1, 2].flat_map(x => [x, x]).collect())
print([3, 1, 4].scan(max).all)
