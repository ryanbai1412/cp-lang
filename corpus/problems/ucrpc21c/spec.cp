// ucrpc21c: two teams start with n health and trade damage each round; report the first round in
// which a team (or both) falls, or that nobody does.
let (n, m) = read<(Int, 1..=100_000)>()
var (a, b) = (n, n)
for round in 1..=m:
    let (u, v, x, y) = read<(Int, Int, Int, Int)>()
    a -= u * v
    b -= x * y
    if a <= 0 or b <= 0:
        print(
            if a <= 0 and b <= 0: "It's a tie at round {round}!" elif a <= 0: "Team 1 wins at round {round}!" else: "Team 2 wins at round {round}!"
        )
        break
else:
    print("Oh no!")
