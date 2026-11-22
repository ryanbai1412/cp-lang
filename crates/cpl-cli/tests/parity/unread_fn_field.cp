struct Acc:
    total: Int
    var hook: ?(Int -> Int)
    step: Int -> Int

struct Tagged:
    tag: Int
    pick: ?(Int -> Int)

fn run(a: Acc, n: Int) -> Int:
    var t = a.total
    for i in 0..n:
        t += a.step(i)
    t

let k = read::<Int>()
let a = Acc(1, none, x => x * k)
var b = Acc(2, none, x => x + 1)
b.hook = x => x + k
print(run(a, 5), run(b, 3))
let t = Tagged(7, x => x - k)
let u = Tagged(8, none)
match t.pick:
    none: print("none")
    some(f): print(f(10))
print(yes(u.pick == none), t.tag + u.tag)
