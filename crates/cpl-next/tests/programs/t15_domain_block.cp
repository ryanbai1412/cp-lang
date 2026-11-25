// expect: ok, output "2\n10\n9\n1002\n1005\n1003"
// input: "3 2 5 3\n"
// A domain declared in a top-level loop: each round has its own, and closures keep theirs.
var fs: List<Int -> Int> = []
let t: Int = read()
for r in 0..t:
    let n: Int = read()
    domain V = 0..n
    var a: [V]Int = r + 1
    fs.push(x => x + V.count)
    var s = 0
    for v in V: s += a[v]
    print(s)
for i in 0..fs.len: print(fs[i](1000))
