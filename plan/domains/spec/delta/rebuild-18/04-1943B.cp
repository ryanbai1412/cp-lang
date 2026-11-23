// CF1943B Non-Palindromic Substring. For k < len every length-k substring is
// a palindrome iff (k odd and t alternates) or (k even and t is constant);
// k = len counts iff t itself is not a palindrome.
@!multitest
let n: 2..=2e5 = read()
let q: 1..=2e5 = read()
let s = read<Str>()
let pal = manacher(s)
var diff: [n + 1]Int = n    // first j >= i + 1 with s[j] != s[j - 1]
var alt: [n + 1]Int = n     // first j >= i + 2 with s[j] != s[j - 2]
for i in (0..n - 1).rev:
    diff[i] = if s[i + 1] != s[i]: i + 1 else: diff[i + 1]
for i in (0..n - 2).rev:
    alt[i] = if s[i + 2] != s[i]: i + 2 else: alt[i + 1]
repeat q:
    let l = read<1..=n>() - 1
    let r = read<1..=n>() - 1
    let len = r - l + 1
    var ans = 0
    if diff[l] <= r: ans += (2..len).step(2).sum()
    if alt[l] <= r: ans += (3..len).step(2).sum()
    if not pal.is_palindrome(l..r + 1): ans += len
    print(ans)
