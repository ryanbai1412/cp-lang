// expect: ok, output "15 7 7 1 2 3 4 15"
// rule: 05.6.3
// A method ending in `lend mut place` returns a writable place the caller assigns through; `lend place` returns a read lend.
struct Grid:
    cols: Int
    data: List<Int>

    fn at(mut self, r: Int, c: Int) -> lends mut Int:
        lend mut data[r * cols + c]

    fn peek(self, r: Int, c: Int) -> lends Int:
        lend data[r * cols + c]

var g = Grid(cols: 3, data: List(0..6))
g.at(1, 2) += 10
g.at(0, 0) = 7
print(g.peek(1, 2), g.peek(0, 0), g.data)
