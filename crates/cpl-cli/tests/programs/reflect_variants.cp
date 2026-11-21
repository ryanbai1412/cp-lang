// `fields(e)` on an enum walks the payload of the variant `e` is (decision 28.6).
enum Op:
    Add(Int, Int)
    Neg(Int)
    Name(Str)
    Stop

fn show_payload(e: Op):
    for f in fields(e):
        print(f)

fn same_payload(a: Op, b: Op) -> Int:
    var n = 0
    for (x, y) in fields(a, b):
        if x == y:
            n += 1
    n

show_payload(Op.Add(1, 2))
show_payload(Op.Neg(5))
show_payload(Op.Name("abc"))
show_payload(Op.Stop)
print(same_payload(Op.Add(1, 2), Op.Add(1, 3)), same_payload(Op.Add(1, 2), Op.Neg(1)), same_payload(Op.Name("x"), Op.Name("x")))
