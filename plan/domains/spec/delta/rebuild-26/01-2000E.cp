// 2000E Photoshoot for Gorillas: a cell's weight is the number of k x k squares
// covering it = the number of valid top-left corners in the k x k block ending at it.
@!multitest
let (n, m, k): (1..=2e5, 1..=2e5, 1..=2e5) = read()
let w: 1..=2e5 = read()
let heights = read<[w]1..=1e9>()
domain R = 0..n
domain C = 0..m
domain Cell = (R, C)
let corner: [R, C]Int = [R, C] (r, c) => if Int(r) + k <= n and Int(c) + k <= m: 1 else: 0
let corners = Prefix(corner)
let cover = [corners.fold(R(max(0, Int(r) - k + 1))..=r, C(max(0, Int(c) - k + 1))..=c) for (r, c) in Cell]
let tall = heights.sorted(by: h => desc(h))
let busy = cover.sorted(by: x => desc(x))
print(sum(Int(h) * x for (h, x) in zip(tall, busy)))
