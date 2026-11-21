// The open register of the Trap Parity Gate: one site per `which` where a wrong program
// is not yet stopped by both `cpl run` and the judge build. A site that both stop moves
// to trap_parity.cp. Input: `which 0 3`.
let (which, zero, three): (Int, Int, Int) = read()
domain D = 0..three

// 1 to 4: `Int` overflow stops `cpl run`; the judge build checks it only under `-DLOCAL`.
if which == 1: print(Int.MAX + three)
if which == 2: print(Int.MAX * three)
if which == 3: print(2 ** (61 + three))
if which == 4: print(zero - three - Int.MAX - Int.MAX)

// 5: `Fenwick.kth` over a negative entry that its walk never visits answers 2 in both; the first
// prefix sum that reaches 1 is at 0.
if which == 5:
    var f = Fenwick<D, Int>()
    f.add(D(0), 1)
    f.add(D(1), -1)
    f.add(D(2), 3)
    print(f.kth(1))
