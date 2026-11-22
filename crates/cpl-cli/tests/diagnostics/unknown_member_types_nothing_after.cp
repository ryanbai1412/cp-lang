domain V = 0..3
let order = [V] v => v
let pos = order.invers()
let rank: [V]Int = [V] u => pos[u] + 1
print(rank)
