// expect: ok, output "103 0 1 3 103\n113\n39"
// rule: 06.2.1
// Closures that write a captured `var` see each other's writes and the program's, when passed to a fn and when one is made inside another.
var cnt = 0
var hist: List<Int> = []
let bump = (d: Int) => cnt += d
let rec = () => hist.push(cnt)
for i in 0..4:
    bump(i)
    rec()
    if i == 2:
        cnt = 100
print(cnt, hist)
fn twice(g: Int -> (), x: Int):
    g(x)
    g(x)
twice(bump, 5)
print(cnt)
var acc = 0
let outer = (k: Int) =>
    let inner = (j: Int) => acc += j * k
    inner(1)
    inner(2)
outer(3)
outer(10)
print(acc)
