// expect: E0000
// (pending) a diagnostic code for: instantiation does not end
// input: "8\n"
fn solve(xs: List<Int>, depth: Int, f: Int -> Int) -> Int:
    if xs.len <= 1 or depth > 3: return f(xs[0])
    let first = xs[0]
    let g = (x: Int) => f(x) + first
    var half: List<Int> = []
    for i in 0..xs.len:
        if i % 2 == 0: half.push(xs[i])
    solve(half, depth + 1, g)
let m: Int = read()
var xs: List<Int> = []
for i in 0..m: xs.push(i + 1)
print(solve(xs, 0, x => x * 2))
