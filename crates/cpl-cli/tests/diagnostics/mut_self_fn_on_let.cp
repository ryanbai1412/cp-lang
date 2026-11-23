fn rotate(mut self: List<Int>):
    self.swap_at(0, 1)
let a = [1, 2, 3]
a.rotate()
print(a)
