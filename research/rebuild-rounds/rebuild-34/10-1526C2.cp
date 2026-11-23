// CF 1526C2 Potions: drink everything; whenever health goes negative, undo
// the most harmful potion drunk so far.
let n: 1..=2e5 = read()
let a = read<[n]-1e9..=1e9>()

var drunk: Heap<Int> = Heap()
var health = 0
for v in a:
    health += v
    drunk.push(v)
    if health < 0: health -= drunk.pop()!
print(drunk.len)
