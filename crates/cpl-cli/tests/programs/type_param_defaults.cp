struct Pair<A, B = Int>(a: A, b: B)

let p: Pair<Int> = Pair(3, 4)
print(p.a + p.b)

let q: Pair<Int, Int> = p
print(q.a * q.b)

let r = Pair<Str>("hi", 7)
print(r.a, r.b)
