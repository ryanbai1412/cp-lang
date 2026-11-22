// expect: ok, output "7 3"
// rule: 01.2.2
fn app(x: Int, f: Int -> Int) -> Int = f(x)
let r = app(3, x =>
    let y = x * 2
    y + 1
)
let t = (1,
            2)
print(r, t.0 + t.1)
