fn apply(f: Int -> Int, x: Int) -> Int = f(x)

let n: Int = read()
var k = 3
let add = (x: Int) => x + k
let pick = (x: Int) => if x % 2 == 0: Some(x) else: None
var s = 0
for i in 0..n:
    s += add(i)
    if let Some(v) = pick(i): s += v
print(s, apply(add, 10))
let show = (x: Int) => print("got", x)
show(s)
