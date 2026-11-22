// A closure inside a free function written with a `self` parameter
// captures that `self`, as one inside a method does.
fn count_over(self: List<Int>, t: Int) -> Int = self.len - ((0..self.len).first_true(i => self[i] > t) ?? self.len)

fn scaled(self: List<Int>, k: Int) -> List<Int> = (0..self.len).map(i => self[i] * k).to_list()

let xs = [1, 3, 5, 7, 9]
print(xs.count_over(4))
print(xs.count_over(9))
print(xs.scaled(2))
