// ETSR 2023 R1 P4: range add, range floor-divide, range min, range sum. Floor division is a beats
// update: it recurses until a node's min and max move by the same amount, then it is an add.
let (n, q): (1..=2e5, 1..=2e5) = read()
domain I = 1..=n
let a: [I]-2147483648..=2147483647 = read()
var t = BeatsSegTree([I] i => Int(a[i]))
for _ in 0..q:
    let (op, l, r): (1..=4, I, I) = read()
    match op:
        1 => t.add(l..=r, read<-1e4..=1e4>())
        2 => t.chdiv(l..=r, read<1..=1e9>())
        3 => print(t.min_val(l..=r)!)
        _ => print(t.sum(l..=r))
