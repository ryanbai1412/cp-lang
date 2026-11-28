// expect: E0603
// rule: R05.10.4
// A closure argument is a parameter of the fn it is passed to: that fn may call it, or pass it on
// to be called, but pushing it into a `mut` parameter keeps it past the call.
fn apply(f: (Int) -> Int, mut out: List<Int>): out.push(f(1))
fn keep(f: (Int) -> Int, mut out: List<(Int) -> Int>):
    let g = f
    out.push(g) // <- here
fn run():
    var xs: List<Int> = []
    apply(x => x + 1, mut xs)
    var fs: List<(Int) -> Int> = []
    keep(x => x * 2, mut fs)
    print(xs.len + fs.len)
run()
