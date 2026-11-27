// expect: E0603
// rule: R05.7.8
// `var out = fs` keeps the caller's lent list, so returning it returns a lend of `fs`; a lent heap
// parameter copied into a `var` is `fs.clone()` (spec 5, "Binding a lent value").
fn add(fs: List<Int -> Int>, k: Int) -> List<Int -> Int>:
    var out = fs
    out.push(x => x + k)
    out // <- here
var start: List<Int -> Int> = []
let fs = add(start, 5)
print(fs[0](1))
