// SAC 22 CC4 J2: set slot i to a shape, or ask whether slot i holds a shape.
let (n, q) = read<(1..=100_000, 1..=100_000)>()
domain I = 1..=n
enum Shape = @tag("square") Square | @tag("circle") Circle | @tag("triangle") Triangle
enum Q = @tag("set") Put(s: Shape, i: I) | @tag("ask") Ask(s: Shape, i: I)
var slot: [I]?Shape = None
for query in read<[q]Q>():
    match query:
        Put(s, i) => slot[i] = s
        Ask(s, i) => print(Int(slot[i] == s))
