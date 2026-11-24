// expect: E0606
// rule: 08.1.6
struct F: f: Int -> Int
let a = F(x => x)
print(yes(a == a)) // <- here
