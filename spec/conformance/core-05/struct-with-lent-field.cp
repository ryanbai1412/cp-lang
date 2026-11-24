// expect: E0602
// rule: 05.7.2
struct Holder: xs: List<Int>
fn f(mut xs: List<Int>):
    let h = Holder(xs)
    xs.push(1) // <- here
    print(h.xs.len)
