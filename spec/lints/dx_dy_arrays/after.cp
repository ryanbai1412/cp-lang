let (h, w): (1..=1000, 1..=1000) = read()
domain R = 0..h
domain C = 0..w
let (x, y): (R, C) = read()
for d in Dir4.all():
    if let (nx, ny) = (x, y).step(d):
        print(nx, ny)
