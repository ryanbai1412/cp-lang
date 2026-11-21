let n: Int = read()
let target: [0..n]?Int = read(none: -1)
print(target[0] ?? -9, target[1] ?? -9, target[2] ?? -9)

fn wrap(x: Int) -> Int: x + 1
let v = wrap(match n:
    3 => 41
    _ => 0
)
print(v)

struct Box3:
    cells: [0..3]Int = 7

let b = Box3()
print(b.cells[0], b.cells[1], b.cells[2])
