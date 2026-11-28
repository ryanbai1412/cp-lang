// expect: ok, output "ok"
// model: samples_of
use std.laws.{Sample, samples_of}

enum Color:
    Red
    Green
    Blue

type Small = 1..=9

fn all<T: Sample>() -> List<T> = T.samples()

/// A long range's samples, by brute force: each Int sample inside it and its two first and two
/// last values, sorted, each once.
fn brute(lo: Int, hi: Int) -> List<Int>:
    var xs = List<Int>()
    for x in all<Int>():
        if lo <= x and x <= hi: xs.push(x)
    for x in [lo, lo + 1, hi - 1, hi]: xs.push(x)
    xs.sort()
    xs.dedup()
    xs

let big = all<-5..=1000000>()
let want = brute(-5, 1000000)
assert big.len == want.len
for i in 0..big.len: assert big[i] == want[i]
let small = all<0..=3>()
assert small.len == 4
for i in 0..small.len: assert small[i] == i
assert all<Int>().len == 13 and all<Int>()[0] == 0
assert all<Bool>().len == 2 and not all<Bool>()[0] and all<Bool>()[1]
assert all<Char>()[0] == 'a' and all<Str>()[0] == "" and all<F64>()[5] == -3.25
// A domain's samples are its positions, in walk order; a range's own impl wins over the domain's.
let colors = samples_of<Color>()
assert colors.len == 3 and colors[0] == Color.Red and colors[2] == Color.Blue
assert samples_of<Small>().len == 9 and samples_of<Small>()[0] == 1
assert samples_of<Int>().len == 13
print("ok")
