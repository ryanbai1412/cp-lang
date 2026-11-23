// CF 1916D Mathematical Problem (any answer). 14^2 = 196, (10^(i+1) + 3)^2 =
// 1 0^i 6 0^i 9 and (3 * 10^(i+1) + 1)^2 = 9 0^i 6 0^i 1 all use the digits
// {1, 6, 9} plus zeros; padding with an even number of zeros keeps a square.
@!multitest
let n: 1..=99 = read()
fn z(k: Int) -> Str = "0".repeat(k)
if n == 1:
    print(1)
else:
    print("196" + z(n - 3))
    for i in 0..=(n - 3) / 2:
        let pad = z(n - 2 * i - 3)
        print("1" + z(i) + "6" + z(i) + "9" + pad)
        print("9" + z(i) + "6" + z(i) + "1" + pad)

@gen
fn input():
    let t = gen(1..=5)
    print(t)
    for _ in 0..t: print(2 * gen(0..=20) + 1)

fn is_square(x: BigInt) -> Bool:
    let one = BigInt.from_int(1)
    let two = BigInt.from_int(2)
    var lo = BigInt.from_int(0)
    var hi = one
    while hi * hi <= x: hi = hi * two
    while hi - lo > one:
        let mid = (lo + hi) / two
        if mid * mid <= x: lo = mid else: hi = mid
    lo * lo == x

@check
fn ok(input: Reader, out: Reader) -> Bool:
    let t: Int = input.read()
    for _ in 0..t:
        let n: Int = input.read()
        var seen: Set<Str> = Set()
        var digits = ""
        for _ in 0..n:
            let s: Str = out.read()
            if s.len != n or s[0] == '0' or s in seen: return false
            seen.add(s)
            let key = Str(s.sorted())
            if digits == "": digits = key
            elif digits != key: return false
            if not is_square(BigInt.from_str(s)): return false
    true
