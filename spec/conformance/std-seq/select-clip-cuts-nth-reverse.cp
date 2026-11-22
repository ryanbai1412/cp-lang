// expect: ok, output "YES\n0 1 2 3 4 5 6 7 8 9 \n3\nYES YES\nYES YES\nYES\nYES\nYES\nYES YES YES\nYES NO YES\nYES\nYES YES"
var a = [5, 1, 4, 2]
a.select(1)
print(a.nth(1) == 2)
var b = [9, 3, 7, 1, 8, 2, 6, 4, 5, 0]
for k in 0..10:
    var c = b.copy()
    c.select(k)
    print(c.nth(k), end: " ")
print()
var d = [3, 1, 2]
d.select(0, by: x => -x)
print(d[0])
print((2..5).l == 2, (2..5).r == 5)
print((0..5).clip(3..9) == 3..5, (0..5).clip(6..9).is_empty)
print([5, 6, 7][1..].nth(0) == 6)
var e = [1, 2, 3, 4]
e.reverse()
print(e == [4, 3, 2, 1])
reverse(mut e, range: 1..3)
print(e == [4, 2, 3, 1])
print([3, 9, 2, 9].argmax() == Some(1), [3, 1, 1].argmin() == Some(1), [3, 9, 2, 9].argmax(by: x => -x) == Some(2))
print("ace".is_subsequence("abcde"), "aec".is_subsequence("abcde"), [1, 3].is_subsequence([1, 2, 3]))
let f = [1]
var g = f.clone()
g.push(2)
print(f == [1])
print((0..100).bsearch(x => x * x >= 50) == 8, (0..100).bsearch(x => x * x <= 50, last: true) == 8)
