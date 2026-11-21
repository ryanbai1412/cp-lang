enum Dir: N, E, S, W
enum Q: ask(Int), answer(Int)
var best = 0, ways: Int = 1
let a = +3, b = -best
let qs = [Q.ask(2), Q.answer(5)]
for q in qs:
    match q:
        Q.ask(x) => best += x
        Q.answer(x) => ways *= x
print(best, ways, a + b, (1, +1))
