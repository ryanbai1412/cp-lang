// Prices fall strictly and only P[0] is known. Paying M buys greedily from type 0,
// so a transaction reports a set of types and their total price: one equation whose
// smallest unknown type is the first one M can afford. Keep one equation per smallest
// unknown. Scanning from the right: just past a known price, pay one less than it; an
// equation in one unknown solves it; otherwise pay the equation's average, less than
// its first price, which yields an equation for a later type. No type is bought more
// than its quota, so finish by topping each type up to exactly i.
//
// Interaction (interactor.py): read N and P[0]; a transaction prints "? M" and reads
// the k types bought and the change; "!" ends.
let n = read<2..=100>()
domain T = [n]
let p0 = read<1..=1e15>()

fn transaction(m: Int) -> (List<T>, Int):
    print("?", m)
    let k = read<1..=n>()
    let types: List<T> = read<[k]T>()
    (types, read<Int>())

struct Eq:
    var types: Bitset<T>
    var sum: Int

fn buy_souvenirs(p0: Int):
    var price: [T]?Int = None
    var bought: [T]Int = 0
    var eq: [T]?Eq = None
    price[T.first!] = p0

    fn pay(m: Int):
        let (types, change) = transaction(m)
        for x in types: bought[x] += 1
        let (known, unknown) = types.partition(x => price[x] != None)
        eq[unknown.min()!] = Eq(
            types: Bitset.of(unknown),
            sum: m - change - known.sum(of: x => price[x]!),
        )

    fn learn(i: T, v: Int):
        price[i] = v
        eq[i] = None
        for k in T if eq[k] is Some(e) and i in e.types:
            let e = mut eq[k]!
            e.types.remove(i)
            e.sum -= v

    while T.any(i => price[i] == None):
        for i in T.rev:
            if let v = price[i]:
                if T.after(i).any(j => price[j] == None): pay(v - 1); break
            elif let e = eq[i]:
                if e.types.len == 1: learn(i, e.sum) else: pay(e.sum / e.types.len)
                break

    for i in T: repeat i.ord - bought[i]: transaction(price[i]!)

buy_souvenirs(p0)
print("!")
