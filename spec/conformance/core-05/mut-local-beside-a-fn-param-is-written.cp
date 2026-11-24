// expect: ok, output "6 NO YES"
// rule: 05.3.3
// A scalar local lent `mut` to a fn that is also handed a fn value takes the callee's write, as any `mut` place does.
fn onto_i(mut old: Int, new: Int, compose: (Int, Int) -> Int):
    old = compose(new, old)
fn onto_b(mut old: Bool, new: Bool, compose: (Bool, Bool) -> Bool):
    old = compose(new, old)
fn locals(add: (Int, Int) -> Int, ne: (Bool, Bool) -> Bool):
    var i = 1
    onto_i(mut i, 5, add)
    var b = true
    var c = b
    onto_b(mut b, true, ne)
    print(i, yes(b), yes(c))
locals((x, y) => x + y, (x, y) => x != y)
