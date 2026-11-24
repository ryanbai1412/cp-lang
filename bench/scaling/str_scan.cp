// n: 512
// fixed: A
let n: Int = read()
let ss = [Str([Char(Int('a') + (i * 7919 + i / 3) % 3) for i in 0..n]) for _ in 0..2]
let abs = ["ab".repeat(n / 2) for _ in 0..2]
let half = abs[0][0..n / 2]
let wild = Str([if i % 2 == 0: 'a' else: '?' for i in 0..n / 8])
var acc = 0
for r in 0..4096:
    let (s, ab) = (ss[r % 2], abs[r % 2])
    acc += z_function(s)[1] // timed: strings.z_function
    acc += prefix_function(s)[n - 1] // timed: strings.prefix_function
    for k in find_all(ab, half): acc += k // timed: strings.find_all
    acc += period(s) // timed: strings.period
    for b in borders(ab): acc += b // timed: strings.borders
    acc += ab.prefix_matches(half)[n - 1] // timed: strings.prefix_matches
    acc += overlap(ab, abs[1 - r % 2]) // timed: strings.overlap
    for k in ab.find_all_wildcard(wild): acc += k // timed: strings.find_all_wildcard
    acc += manacher(s).len // timed: strings.manacher
print(acc)
