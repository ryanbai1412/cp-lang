// cp: 0.1 88018bf+dirty
let n: 1..=9 = read()
domain V = 1..=n
domain W = 1..=n

fn count_true(s: [D]Bool) -> Int:
    count(s[i] for i in D)

fn first_v(s: [V]Bool) -> Int:
    if s[V.first]: 1 else: 0

let a: [V]Bool = [V] v => v.idx % 2 == 0
let b: [W]Bool = [W] w => w.idx < 2
print(count_true(a), count_true(b), first_v(a))
