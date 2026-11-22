// A method that lends a place makes its call writable: `+=`, `=`, swap and a mutating method go through it.
domain V = 1..=3
struct Board:
    cells: [V]Int
    names: List<Str>
    fn at(mut self, i: V) lends mut Int:
        mut self.cells[i]
    fn name(mut self, i: Int) lends mut Str:
        mut self.names[i]
    fn peek(self, i: V) lends Int:
        self.cells[i]

var b = Board([V] v => 0, ["a", "b"])
b.at(V(1)) += 10
b.at(V(2)) = 5
b.at(V(3)) = b.at(V(1)) + b.at(V(2))
b.at(V(2)) *= 3
b.name(1) = b.name(0) + "!"
for i in V:
    b.at(i) += 1
let p = b.peek(V(3))
print(b.cells, p, b.names)
