let (n, x) = read<(1..=2e5, 1..=1e9)>()
domain I = 0..n
let a = read<[I]1..=1e9>()
let p = Prefix(a)
fn ends_at(c: Cut<I>) -> Bool:
    if let Some(d) = p.first_ge(p.prefix(c) + x):
        return p.prefix(d) == p.prefix(c) + x
    false
print(I.cuts.count(c => ends_at(c)))
