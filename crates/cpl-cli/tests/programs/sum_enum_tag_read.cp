// A payload enum over domains is a sum domain, and it reads by its tag
// (R10.2.3), alone or inside a tuple: `3 2` is `Del(2)`, not position 3.
let (n, q_cnt): (1..=5, 1..=5) = read()
domain Pos = 1..=n
domain Query = 1..=q_cnt
enum Op:
    @tag(1) Ins(r: Pos)
    @tag(3) Del(id: Query)
for _ in 0..2:
    let (op, p): (Op, Pos) = read()
    match op:
        Ins(r) => print("ins", r, p)
        Del(id) => print("del", id, p)
let last: Op = read()
match last:
    Ins(r) => print("ins", r)
    Del(id) => print("del", id)
