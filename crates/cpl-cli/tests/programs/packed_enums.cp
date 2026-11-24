// A list or array of enum values with a few scalar fields is stored packed;
// reading, writing, sorting and matching it behaves as on boxed values.
enum Ask:
    Set(i: Int, x: Int)
    Sum(l: Int, r: Int)
    Flip(on: Bool)
    Stop
var asks: List<Ask> = []
for i in 0..6:
    if i % 3 == 0:
        asks.push(Set(i, i * i))
    elif i % 3 == 1:
        asks.push(Sum(i - 1, i))
    else:
        asks.push(Flip(i % 2 == 0))
asks.push(Stop)
asks[1] = Set(9, -9)
var total = 0
for ask in asks:
    match ask:
        Set(i, x) => total += i + x
        Sum(l, r) => total += l * r
        Flip(on) => total += if on: 1 else: 0
        Stop => total -= 1
print(total, asks.len)
var grid: [3]Ask = Stop
grid[0] = Sum(1, 2)
grid[2] = asks[0]
print(yes(grid[0] == Sum(1, 2)), yes(grid[1] == Stop), yes(grid[2] == Set(0, 0)))
let last = asks.pop()
print(yes(last == Stop), asks.len)
