// Codeforces 2152 C — Triple Removal on Binary Subarrays
//
// Each operation removes three equal elements from the subarray `[l..r]`, so
// the subarray is clearable iff both its length `len = r - l` and its count of
// `1`s (`cnt_a.fold(l..r)`) are divisible by 3 (equivalently both `0`s and `1`s
// occur in multiples of 3).
//
// `cnt_chk` is the prefix sum of `a[i] ^ (i & 1)`, which is constant (`0` or
// `len`) on `[l..r]` iff the subarray strictly alternates (`0101...` or
// `1010...`). Every triple costs 1 if an adjacent equal pair exists, while a
// strictly alternating subarray must pay cost 2 on its first removal (adjacent
// identical elements are 2 apart) and 1 on every remaining triple, giving
// `len / 3 + Int(min(raw, len - raw) == 0)`.

@!multitest
let (n, q): (1..=200_000, 1..=200_000) = read()
domain Pos = 1..=n
let a: [0..n]0..=1 = read()
let cnt_a = prefix(a)
let cnt_chk = prefix([a[i] ^ (i & 1) for i in 0..n])

for _ in 0..q:
    let (l_pos, r_pos): (Pos, Pos) = read()
    let l = l_pos.ord
    let r = r_pos.ord + 1
    let len = r - l
    if len % 3 != 0 or cnt_a.fold(l..r) % 3 != 0:
        print(-1)
    else:
        let raw = cnt_chk.fold(l..r)
        print(len / 3 + Int(min(raw, len - raw) == 0))
