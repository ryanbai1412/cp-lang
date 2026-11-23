// CF 1526C2 revised: you start with health h (read after n) and print the
// most potions and the largest final health reaching that count.
// Drink everything; whenever health goes negative, undo
// the most harmful potion drunk so far.
let n: 1..=2e5 = read()
let h: 0..=1e9 = read()
let a = read<[n]-1e9..=1e9>()

var drunk: Heap<Int> = Heap()
var health = h
for v in a:
    health += v
    drunk.push(v)
    if health < 0: health -= drunk.pop()!
print(drunk.len, health)
