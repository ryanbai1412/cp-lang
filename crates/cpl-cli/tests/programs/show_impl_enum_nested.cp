enum Dir:
    Up
    Down
impl Show for Dir:
    fn show(self) -> Str = match self:
        Up => "U"
        Down => "D"
impl Dir:
    fn flip(self) -> Dir = match self:
        Up => Down
        Down => Up
let d = Dir.Up
print(d, Dir.Down, "{d}")
print(d.flip(), d.show())
print([d, Dir.Down], sep: " ")
print((d, 1))
struct P:
    x: Int
impl Show for P:
    fn show(self) -> Str = "P"
print((P(x: 1), 1))
print([P(x: 1)], sep: " ")
