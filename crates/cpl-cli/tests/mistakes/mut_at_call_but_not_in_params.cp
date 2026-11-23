// here: 4
// The caller says `mut` but the parameter does not (acceptance run): one error, in the fn, not a second at the call.
fn add_one(a: List<Int>):
    a.push(1)
var b = [1]
add_one(mut b)
print(b)
