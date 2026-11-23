struct Doubled(pub xs: List<Int>):
    pub fn at(self, i: Int) -> Int = 2 * xs[i]
let a = [10, 20, 30, 40, 50]
let e = a.step(2)
print(e[2], e[4], a.even[0])
print("abcde".step(2)[2])
let d = Doubled([3, 4])
print(d[1] + d[0])
