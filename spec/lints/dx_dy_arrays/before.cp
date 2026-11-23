let (h, w): (1..=1000, 1..=1000) = read()
let (x, y): (0..h, 0..w) = read()
let dx = [1, -1, 0, 0]
let dy = [0, 0, 1, -1]
for d in 0..4:
    print(x + dx[d], y + dy[d])
