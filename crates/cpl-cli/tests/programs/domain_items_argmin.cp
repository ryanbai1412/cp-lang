// A domain's `.items` are its range's, keyed by the elements (R04.4.31),
// so `for (k, v) in I.items` walks and std's `I.argmin`/`I.argmax`
// (generic over `S: Keyed`) return an `I`.
let n = read<1..=9>()
domain I = 0..n
for (k, v) in I.items: print(k, v)
let a: [I]Int = [I] i => (i.ord - 2) * (i.ord - 2)
print(I.argmin(i => a[i])!, I.argmax(i => a[i])!)
print([k for (k, v) in I.all.items if a[v] == 1])
for (k, v) in (2..4).items: print(k, v)
