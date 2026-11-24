// expect: ok, output "1 2 1 2 1 2\n5 5 6"
// rule: 05.3.1
// A clone in one fn and a `let` of a `var` in another stay separate values, and so does a `var` copy of a map entry, however codegen reuses slots across fns.
fn first(xs: List<Int>) -> List<Int>:
    let a = xs
    var w = a.clone()
    w.push(99)
    return a
fn second(ys: List<Int>) -> List<Int>:
    var v = ys
    let t = v.clone()
    v.push(7)
    return t
let base = [1, 2]
print(first(base), second(base), base)
var m: Map<Int, List<Int>> = Map()
m[1] = [5]
var w = m[1]
w.push(6)
print(m[1], w)
