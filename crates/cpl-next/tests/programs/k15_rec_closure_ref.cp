// expect: ok, output "90 1"
// Each recursive activation has its own var; a closure passed down writes the right one.
fn rec(n: Int, f: Int -> ()) -> Int:
    var mine = n
    let g = (x: Int) => mine += x
    if n > 0:
        let r = rec(n - 1, g)
        f(1)
        return mine * 10 + r
    f(1)
    mine
var top = 0
print(rec(3, x => top += x), top)
