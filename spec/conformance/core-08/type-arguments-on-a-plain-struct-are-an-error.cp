// expect: E0303
// rule: 08.2.5
// `Pt` declares no generics, so `Pt<1..=100>` names nothing; the argument is not silently dropped.
struct Pt:
    x: Int
    y: Int
let p: Pt<1..=100> = Pt(500, 3) // <- here
print(p.x)
