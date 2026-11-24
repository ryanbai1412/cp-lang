// Source: IOI 2025 Day 1 (Souvenirs), https://github.com/ioi/task-archive/tree/master/2025/day1/souvenirs
// limits: 1000 ms, 2048 MB
// Problem: interactive (function task). N souvenir types have strictly falling prices
//   P[0] > ... > P[N-1] >= 1, only P[0] known. transaction(M), for P[0] > M >= P[N-1],
//   buys greedily from type 0 (one of each type M still covers) and returns the types
//   bought and the change. Buy exactly i souvenirs of type i in at most 5000 transactions.
// Solution: a transaction gives one equation (the types bought and their total price) whose
//   smallest unknown type is the first one M can afford; keep one per smallest unknown, as a
//   Bitset of unknown types and their sum. Scan from the right: just past a known price, pay
//   one less than it; an equation in one unknown solves it (and is peeled off the others);
//   otherwise pay the equation's average, which yields an equation for a later type. No type
//   is bought more than i times, so top each type up to i at its known price.
// Interaction (interactor.py, the official manager's protocol over stdio): read N and P[0];
//   transaction(M) prints "? M" and reads the k types bought and the change; "!" ends.
// original: corpus/problems/ioi25_souvenirs/orig.cpp (the official model solution plus the sample grader)
// Bounds: n: 2..=100; p0: 1..=1e15; k: 1..=n

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
