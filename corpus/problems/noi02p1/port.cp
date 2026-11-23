// cp: 0.1 8ef64b9+dirty
// NOI 2002 Galactic Heroes. Every ship starts as its own column.
// Merge(a, b) moves a's whole column behind b's; Ask(a, b) prints how many
// ships stand between a and b, or -1 if they are in different columns.
domain Ship = 1..=30000

@tags("M", "C")
enum Op:
    Merge(a, b: Ship)
    Ask(a, b: Ship)

let q: 1..=5e5 = read()
domain Query = 0..q

// few distinct ships, so columns actually meet
fn random_op() -> Op:
    let (a, b) = (Ship(rand(1, 5)), Ship(rand(1, 5)))
    if rand(0, 1) == 0: Merge(a, b) else: Ask(a, b)

// ops are handled as they are read, never stored
fn ops() -> yields Op:
    for _ in Query:
        let op: Op = read(gen: random_op)
        yield op

var toward: [Ship]?Ship = none  // a ship nearer the front of the same column
var gap: [Ship]Int = 0          // distance from u to toward[u]
var size: [Ship]Int = 1         // column size, valid at front ships

// (front ship of u's column, ships in front of u)
fn locate(u: Ship) -> (Ship, Int):
    if let p = toward[u]:
        let (f, d) = locate(p)
        gap[u] += d
        toward[u] = f
        return (f, gap[u])
    (u, 0)

for op in ops():
    match op:
        Merge(a, b) =>
            let (fa, da) = locate(a)
            let (fb, db) = locate(b)
            if fa != fb:
                toward[fa] = fb
                gap[fa] = size[fb]
                size[fb] += size[fa]
        Ask(a, b) =>
            let (fa, da) = locate(a)
            let (fb, db) = locate(b)
            print(if fa == fb: abs(da - db) - 1 else: -1)
