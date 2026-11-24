// expect: ok, output "1 3 5\n7\n7"
// rule: 04.4.24
// A `List<0..50>` is no wider than `0..100`, so it moves and extends as it is; a checked comprehension takes a `List<Int>`.
fn f(xs: List<0..100>) -> 0..100 = xs[0]
fn mk(zs: List<Int>) -> List<0..100> = [z for z in zs]
var ys: List<0..100> = [1]
let ok: List<0..50> = [3]
ys.extend(ok)
let zs: List<Int> = [5]
ys.extend([x for x in zs])
print(ys)
let ws: List<0..50> = [7]
print(f(ws))
print(mk([7])[0])
