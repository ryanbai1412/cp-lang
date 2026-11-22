// expect: ok, output "3 2"
domain Row = 0..2
domain Col = 0..3
domain Cell = (Row, Col)
fn moves(p: Cell) -> yields Cell:
    for d in Dir4.all():
        if let q = p.step(d): yield q
print(moves(Cell((0, 1))).count(), moves(Cell((0, 0))).count())
