struct Best:
    @cache top: Int
    xs: List<Int>

    fn top_value(self) -> Int:
        if self.top < 0: self.top = self.xs.max()!
        self.top

var b = Best(top: -1, xs: [3, 9, 4])
b.top = 0
print(b.top_value())
