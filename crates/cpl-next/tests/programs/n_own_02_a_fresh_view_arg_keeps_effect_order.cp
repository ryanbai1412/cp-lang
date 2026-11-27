// expect: ok, output "loud\nmade\n4\nmade\n3"
// rule: R11
// A fresh value given to a param the callee only views is set in a temporary the caller
// releases. An earlier operand with an effect goes to a temporary first, so `loud` still
// prints before `made`; a later operand reading a value the call views reads it unchanged.
fn loud(x: Int) -> Int:
    print("loud")
    x
fn made(n: Int) -> List<Int>:
    print("made")
    [k for k in 0..n]
fn total(n: Int, xs: List<Int>) -> Int = n + xs.len
print(total(loud(1), made(3)))
let ys = made(2)
print(total(ys.len - 1, ys) + 0 * ys[0])
