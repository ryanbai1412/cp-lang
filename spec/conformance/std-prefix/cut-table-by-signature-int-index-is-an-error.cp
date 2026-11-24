// expect: E0320
// Any type that reads by count with `cut(self, k: Int)` is a cut table, not only std's `Prefix`.
struct Counts:
    p: List<Int>

    fn at(self, i: Int) -> Int = self.p[i]
    fn cut(self, k: Int) -> Int = self.p[k]

let t = Counts([0, 3, 4])
print(t[1]) // <- here
