// expect: ok, output "3 1 2\n3 1 2\n5 4\n8 6 7\n3 1 2"
// rule: R01.6.4
// `(a, b) = (b, a)` builds the right side first, so each part is a copy of its own: writing a
// place a part was read from leaves the part alone, whether the places are owned, `mut`
// params of a heap or a type parameter's type, a struct's fields or std's `swap`.
fn sw(mut a: List<Int>, mut b: List<Int>):
    (a, b) = (b, a)
fn gsw<T>(mut a: T, mut b: T):
    (a, b) = (b, a)
struct P(var x: List<Int>, var y: List<Int>):
    fn flip(mut self):
        (x, y) = (y, x)
var a = [1, 2]
var b = [3]
(a, b) = (b, a)
print(a, b)
(a, b) = (b, a)
sw(mut a, mut b)
print(a, b)
var c = [4]
var d = [5]
gsw(mut c, mut d)
print(c, d)
var p = P(x: [6, 7], y: [8])
p.flip()
print(p.x, p.y)
swap(mut a, mut b)
swap(mut a, mut b)
print(a, b)
