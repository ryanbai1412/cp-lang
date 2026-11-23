// 1676E Eating Queries
// in: t; n q; a_1..a_n (sugar per candy); q lines of x. out: per query, fewest candies with total >= x, or -1.
@!multitest
let (n, q) = read<(1..=1.5e5, 1..=1.5e5)>()
let a = read<[n]1..=1e4>()
let eaten = a.sorted(by: x => desc(x)).scan(+)
for x in read<[q]1..=1e9>():
    print(eaten.first_ge(x).map(c => Int(c)) ?? -1)
