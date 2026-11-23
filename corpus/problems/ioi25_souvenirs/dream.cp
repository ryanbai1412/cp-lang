// Prices fall strictly and only P[0] is known. Paying M buys greedily from type 0,
// so a transaction reports a set of types and their total price: one equation whose
// smallest unknown type is the first one M can afford. Keep one equation per smallest
// unknown. Scanning from the right: just past a known price, pay one less than it; an
// equation in one unknown solves it; otherwise pay the equation's average, less than
// its first price, which yields an equation for a later type. No type is bought more
// than its quota, so finish by topping each type up to exactly i.
@grader(interactive)
fn buy_souvenirs(n: 2..=100, p0: 1..=1e15):
    domain T = 0..n
    struct Eq(types: Bitset<T>, sum: Int)
    var price: [T]?Int = None
    var bought: [T]Int = 0
    var eq: [T]?Eq = None
    price[T.first] = p0

    fn pay(m: Int):
        let (types, change) = transaction(m)
        for x in types: bought[x] += 1
        let (known, unknown) = types.partition(x => price[x] != None)
        eq[unknown.min!] = Eq(Bitset(unknown), m - change - known.sum(by: x => price[x]!))

    fn learn(i: T, v: Int):
        price[i] = v
        eq[i] = None
        for e in eq.values_mut() if i in e.types:
            e.types.remove(i)
            e.sum -= v

    while T.any(i => price[i] == None):
        for i in T.rev:
            if let v = price[i]:
                if T.after(i).any(j => price[j] == None): break pay(v - 1)
            elif let e = eq[i]:
                break if e.types.len == 1: learn(i, e.sum) else: pay(e.sum / e.types.len)

    for i in T: repeat i - bought[i]: transaction(price[i]!)
