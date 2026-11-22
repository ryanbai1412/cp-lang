// sac22cc4jp2: set a slot to a shape, or ask whether a slot holds a shape (1 or 0).
let (n, q) = read<(1..=100_000, 1..=100_000)>()
enum Shape:
    @tag("square") Square
    @tag("circle") Circle
    @tag("triangle") Triangle
var at: [1..=n]?Shape = none
for _ in 0..q:
    let (cmd, s, i) = read<(Str, Shape, 1..=n)>()
    if cmd == "set":
        at[i] = s
    else:
        print(Int(at[i] == s))
