// apio09p1: a read() table with a ranged element type is stored at that range's width, 2 B a cell
// here, as in the matched C++, also when it is handed to a fn that only reads it.
let r: 1..=3000 = read()
let c: 1..=3000 = read()
fn weigh(oil: [0..r, 0..c]Int) -> Int:
    var s = 0
    for i in 0..r:
        for j in 0..c: s += oil[i, j] * (i + j)
    return s
let oil: [0..r, 0..c]0..=500 = read()
print(weigh(oil))
