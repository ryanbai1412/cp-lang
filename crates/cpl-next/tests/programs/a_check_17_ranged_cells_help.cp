// expect: E0302
// rule: 04.4.24
// A list of `Int` is not a list of `0..10`: the help checks each element into the range.
fn g() -> List<Int> = [3, 50]
let ys: List<0..10> = g()  // <- here
print(ys[0])
