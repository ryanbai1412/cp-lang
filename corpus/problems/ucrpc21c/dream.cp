// UCRPC 2021 C: both teams start with n; round i takes u·v from the first counter and x·y from
// the second. Report the first round a counter reaches 0 (both at once is a tie), or "Oh no!".
let (n, m) = read (Int, Int)
domain Round = 1..=m
var left = (one: n, two: n)
for (i, (u, v, x, y)) in read [Round](Int, Int, Int, Int).items:
    left = (one: left.one - u * v, two: left.two - x * y)
    match (left.one <= 0, left.two <= 0):
        (true, true): print("It's a tie at round {i}!"); return
        (true, false): print("Team 1 wins at round {i}!"); return
        (false, true): print("Team 2 wins at round {i}!"); return
        _: pass
print("Oh no!")
