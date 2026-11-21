struct Pair<A, B = Int>(a: A, b: B)
let p = Pair(1, 2)
print(p.a + p.b)
