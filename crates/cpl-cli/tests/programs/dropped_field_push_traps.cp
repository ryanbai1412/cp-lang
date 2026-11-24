// A list only a dropped field holds is not built, but what its pushes
// compute still runs: `x * x` overflows on the second list, as it would if
// `b` were kept.
struct Two:
    pub a: List<Int>
    pub b: List<Int>
fn make(xs: List<Int>) -> Two:
    var a = List<Int>()
    var b = List<Int>()
    for x in xs:
        a.push(x)
        b.push(x * x)
    Two(a:, b:)
let n: 1..=10 = read()
let small = make(read<[n]Int>())
print(small.a.len, small.a[n - 1])
let big = make(read<[n]Int>())
print("not reached", big.a.len)
