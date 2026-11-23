// R02.7.6: record patterns in let, for, match and nested in tuples.
let r = (total: 3, take: 4)
let (take: k, ..) = r
let (total:, take:) = r
print(k, total + take)
let es = [(to: 2, w: 5), (to: 7, w: 1), (to: 9, w: 5)]
var heavy = 0
for (to: u, w: c) in es:
    if c == 5:
        heavy += u
print(heavy)
for (i, (w: c, ..)) in es.items:
    print(i, c)
fn pick(e: (to: Int, w: Int)) -> Int:
    match e:
        (to: 7, ..) => 70
        (to: u, w: c) => u * c
print(es.map(pick))
