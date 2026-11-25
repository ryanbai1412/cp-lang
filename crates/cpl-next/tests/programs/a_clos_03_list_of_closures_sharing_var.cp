// expect: ok, output "2 1"
// rule: R05.10.1, R05.10.2 (may be stored in a local var)
// R05.10.1: two closures made in a loop each see their own loop var (plain let, copied), and share the outer var.
fn go() -> Int:
    var total = 0
    var fs: List<() -> Int> = List.new()
    for i in 0..3:
        fs.push(() => { total += i; i })
    let a = fs[2]()
    let b = fs[1]()
    print(a, b)
    total
let t = go()
