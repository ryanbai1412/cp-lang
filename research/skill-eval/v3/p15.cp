let (n, x) = read<(1..=1000, 1..=1e5)>()
let price = read<[n]1..=1000>()
let pages = read<[n]1..=1000>()
domain Budget = 0..=x
var most = [Budget] c => 0
for i in 0..n:
    for c in Budget.rev:
        if let Some(rest) = c.offset(-price[i]):
            most[c] max= most[rest] + pages[i]
print(most[Budget(x)])
