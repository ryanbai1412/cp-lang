// Source: An Animal Contest 5 P4 (Number Game), https://dmoj.ca/problem/aac5p4
// Problem: a board 1..n (n <= 1e6) with a piece at x; each number 1..n - 1 may be used once
//   to move the piece that far forward or backward, staying on the board. Print the least
//   number of moves M after which no move is possible, then such a sequence as signed
//   distances (+ forward, - backward); a right M with a wrong sequence earns 25%.
// Solution: closed form from the original: 0 for n = 1, n / 2 for even n, and for odd n
//   one more when x's parity disagrees with h = (n - 3) / 2 being 0 or 3 mod 4. It prints
//   the count and then 1..count as the moves, not a valid sequence. O(n).
// original: corpus/submissions/4239900.cpp

let n: 1..=1_000_000 = read()
let x: 0..=1_000_000_000 = read()

if n == 1:
    print(0)
else:
    var ans = n / 2
    if n % 2 == 1:
        let h = (n - 3) / 2
        let flip = h % 4 == 0 or h % 4 == 3
        if flip != (x % 2 == 1):
            ans += 1
    print(ans)
    for i in 1..=ans:
        print(i)
