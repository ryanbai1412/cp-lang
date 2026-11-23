// 1955E Long Inversions
// in: t; per test n, then a binary string of length n. out: max k that can make it all ones, per test
@!multitest
let n: Int = read()
let s: Str = read()
'search: for k in (1..=n).rev:
    var ends: [n + 1]Bool = false
    var flipped = false
    for i in 0..n:
        flipped = flipped != ends[i]
        if (s[i] == '1') == flipped:
            if i + k > n: break
            flipped = not flipped
            ends[i + k] = not ends[i + k]
    else:
        print(k)
        break 'search
