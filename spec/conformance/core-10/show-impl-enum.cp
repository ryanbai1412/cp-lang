// expect: ok, output "U D U"
// rule: 10.3.5
enum Dir:
    Up
    Down
impl Show for Dir:
    fn show(self) -> Str = match self:
        Up => "U"
        Down => "D"
let d = Dir.Up
print(d, Dir.Down, "{d}")
