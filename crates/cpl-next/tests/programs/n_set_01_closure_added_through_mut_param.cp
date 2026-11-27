// expect: ok, output "6 7 6 10 4"
// rule: R05.10.2
// A fn that pushes a closure onto a list it is lent `mut` adds that closure to the caller's list,
// and a fn that passes the list on adds what its callee adds; each call is the closure it pushed.
fn add(mut fs: List<Int -> Int>, k: Int):
    fs.push(x => x + k)
fn twice(mut fs: List<Int -> Int>):
    add(mut fs, 5)
    fs.push(x => x * 10)
var fs: List<Int -> Int> = [x => x + 5]
add(mut fs, 6)
twice(mut fs)
print(fs[0](1), fs[1](1), fs[2](1), fs[3](1), fs.len)
