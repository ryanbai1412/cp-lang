// expect: ok, output "1 3 4"
// input: "4\n"
// rule: 05.2.2
// A fn that copies its parameter whole into a `var` takes it by value, so a temporary
// argument moves in.
fn top(xs: List<Int>) -> List<Int>:
    var ys = List(xs)
    ys.sort()
    ys
print(top([read<Int>(), 1, 3]))
