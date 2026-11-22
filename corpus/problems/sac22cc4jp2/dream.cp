// SAC 22 CC4 J2: set slot i to a shape, or ask whether slot i holds a shape.
let (n, q) = read (Int, Int)
domain I = 1..=n
enum Shape: Square = "square", Circle = "circle", Triangle = "triangle"
enum Q: Set(s: Shape, i: I) = "set", Ask(s: Shape, i: I) = "ask"
var slot: [I]?Shape = None
for query in read [q]Q:
    match query:
        Set(s, i): slot[i] = s
        Ask(s, i): print((slot[i] == Some(s)).int)
