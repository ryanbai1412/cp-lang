struct Best:
    @cache top: Int
    xs: List<Int>

    fn top_value(self) -> Int:
        if self.top < 0: self.top = self.xs.max()!
        self.top

let b = Best(top: -1, xs: [3, 9, 4])
print(b.top_value())
