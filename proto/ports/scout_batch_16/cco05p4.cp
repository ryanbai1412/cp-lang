// CCO 2005 P4 Primed Sequences: for each test, find the shortest run of at
// least two consecutive numbers whose sum is prime (the leftmost such run
// of that length), and print it in the judge's sentence; or say that the
// sequence is anti-primed.
//
// The original sieves every sum up to 10^8 before reading anything. The
// sums that are actually asked about are few, so this tests each one.
// original: corpus/submissions/4665767.cpp
fn is_prime(x: Int) -> Bool:
    if x < 2: return false
    var d = 2
    while d * d <= x:
        if x % d == 0: return false
        d += 1
    true

fn solve():
    let n: Int = read()
    domain I = 0..n
    let a: [I]Int = read()
    var pre = [0]
    for x in a: pre.push(pre.last + x)

    for len in 2..=n:
        for l in 0..=n - len:
            if not is_prime(pre[l + len] - pre[l]): continue
            var line = Str("Shortest primed subsequence is length {len}:")
            for i in l..l + len: line.extend(" {a[I.nth(i)]}")
            print(line)
            return
    print("This sequence is anti-primed.")

let t: Int = read()
for _ in 0..t: solve()
