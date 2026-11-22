// expect: E0601 at 5
// rule: 05.10.3
var a = [1, 2]
fn bump(mut x: Int):
    a[1] = 0
    x += 1
bump(mut a[0])
