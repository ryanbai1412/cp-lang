// cp: 0.1 bea787c+dirty
struct SegNode:
    best: Int
    cnt: Int
    fn merge(self, o: SegNode) -> SegNode:
        return SegNode(max(self.best, o.best), self.cnt + o.cnt)
let a = SegNode(4, 1)
let m = a.merge(SegNode(2, 1))
print(m.best, m.cnt)
