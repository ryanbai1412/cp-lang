use std.seq.{zip_exact}
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
let ys = [i for i in 0..n]
let ls = [[i, i] for i in 0..n / 2]
let os = [if i % 2 == 0: Some(i) else: None for i in 0..n]
let small = (x: Int) => x >= 0 // within: seq.take_while
let early = (x: Int) => x < n / 2 // within: seq.drop_while
let halve = (x: Int) => if x % 2 == 0: Some(x / 2) else: None // within: seq.filter_map
let twice = (x: Int) => [x, x] // within: seq.flat_map
let odd_ = (x: Int) => x % 2 == 1 // within: seq.filter
let inc = (x: Int) => x + 1 // within: seq.map
var s = 0
for r in 0..64:
    for w in xs.windows(3): // timed: seq.windows
        s += w.len // timed: seq.windows
    for c in xs.chunks(4): // timed: seq.chunks
        s += c.len // timed: seq.chunks
    for (a, b) in xs.adjacent: // timed: seq.adjacent
        s += a ^ b // timed: seq.adjacent
    for (p, x) in xs.with_prev: // timed: seq.with_prev
        s += x + (p ?? 0) // timed: seq.with_prev
    for (x, q) in xs.with_next(): // timed: seq.with_next
        s += x + (q ?? 0) // timed: seq.with_next
    for (a, b) in xs.cyclic_pairs(): // timed: seq.cyclic_pairs
        s += a ^ b // timed: seq.cyclic_pairs
    for (a, b) in zip_exact(xs, ys): // timed: seq.zip_exact
        s += a ^ b // timed: seq.zip_exact
    for x in chain(xs, ys): // timed: seq.chain
        s += x // timed: seq.chain
    for x in xs.take_while(small): // timed: seq.take_while
        s += x // timed: seq.take_while
    for x in xs.drop_while(early): // timed: seq.drop_while
        s += x // timed: seq.drop_while
    for x in xs.filter_map(halve): // timed: seq.filter_map
        s += x // timed: seq.filter_map
    for x in xs.flat_map(twice): // timed: seq.flat_map
        s += x // timed: seq.flat_map
    for x in ls.flatten(): // timed: seq.flatten
        s += x // timed: seq.flatten
    for x in os.present(): // timed: seq.present
        s += x // timed: seq.present
    for p in xs.keyed(): // timed: seq.keyed
        s += 1 // timed: seq.keyed
    for x in xs.step(2): // timed: seq.step
        s += x // timed: seq.step
    for x in xs.even: // timed: seq.even
        s += x // timed: seq.even
    for x in xs.odd: // timed: seq.odd
        s += x // timed: seq.odd
    for i in ys.record_highs(): // timed: seq.record_highs
        s += i // timed: seq.record_highs
    for r in xs.runs(): // timed: seq.runs
        s += 1 // timed: seq.runs
    for x in xs.filter(odd_): // timed: seq.filter
        s += x // timed: seq.filter
    for x in xs.map(inc): // timed: seq.map
        s += x // timed: seq.map
    for (i, x) in xs.items: // timed: seq.items
        s += i ^ x // timed: seq.items
    for i in xs.keys: // timed: seq.keys
        s += i // timed: seq.keys
    for x in xs.rev: // timed: seq.rev
        s += x // timed: seq.rev
    for (a, b) in zip(xs, ys): // timed: seq.zip
        s += a ^ b // within: seq.zip
    for r in xs.valleys(): // timed: seq.valleys
        s += 1 // within: seq.valleys
for i in 0..1 << 20:
    let z = zip(xs, ys)
    s += z.first[i % n] // timed: seq.Zipped.first
    s += z.second[i % n] // timed: seq.Zipped.second
print(s)
