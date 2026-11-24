// A product position reads as its parts, as it prints (R10.2.3).
domain R = 1..=3
domain C = 1..=3
domain Cell = (R, C)
let c: Cell = read()
print(c)
let d = read<Cell>()
print(d)
let e: (R, C) = read()
print(e)
let cells = read<[0..2]Cell>()
for x in cells:
    print(x)
