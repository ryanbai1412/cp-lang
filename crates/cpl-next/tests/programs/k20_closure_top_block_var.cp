// expect: ok, output "10"
// A var inside a top-level block, shared by a closure made there.
var total = 0
if true:
    var acc = 0
    let add = (x: Int) => acc += x
    for i in 0..5: add(i)
    total = acc
print(total)
