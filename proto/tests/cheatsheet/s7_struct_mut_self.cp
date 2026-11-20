// cp: 0.1 bea787c+dirty
struct S:
    best: Int
    fn reset(mut self): self.best = 0
var a = S(4)
a.reset()
print(a.best)
