// NOIP '19 P1 - Gray Code (DMOJ: noip19p1)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// The n-bit Gray code list is built by reflection: the (n-1)-bit list with a
// 0 in front, then the (n-1)-bit list reversed with a 1 in front. Print the
// k-th n-bit code word (0-indexed), n <= 64 and 0 <= k < 2^n.
//
// The k-th word is k ^ (k >> 1). k does not fit a signed 64-bit integer.
let (n, k): (1..=64, U64) = read()
assume n == 64 or k < 1 << n

let g = k ^ (k >> 1)
print(join(g.bit(i) for i in (0..n).rev))        // WISH: `g.bits(n)`, most significant first

@brute
fn slow() -> Str
    where n <= 12:
    var words = [""]
    for _ in 0..n:
        words = ["0" + w for w in words] + ["1" + w for w in words.rev]
    words[k]
