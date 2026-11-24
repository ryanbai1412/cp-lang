let h: 1..=1000 = read()
let w: 1..=1000 = read()
let moves: Str = read()
var total = 0
for b in 0..200:
    let (br, bc) = (b % h, b * 7 % w)
    var (r, c) = (0, 0)
    for move in moves:
        var (nr, nc) = (r, c)
        if move == '<': nc -= 1
        elif move == '>': nc += 1
        elif move == '^': nr -= 1
        else: nr += 1
        if 0 <= nr < h and 0 <= nc < w and not (nr == br and nc == bc): (r, c) = (nr, nc)
    total += r + c
print(total)
