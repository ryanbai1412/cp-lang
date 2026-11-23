// cf1326d2 Prefix-Suffix Palindrome (hard): strip the matching ends, then take the
// longest palindromic prefix or suffix of the middle via prefix_function on mid#rev(mid).
use std.strings.{prefix_function}

@!multitest
let s = read<Str>()
let n = s.len
var l = 0
while 2 * (l + 1) <= n and s[l] == s[n - 1 - l]: l += 1
let mid = s[l..n - l]
let pre = prefix_function(Str(chain(chain(mid, "#"), mid.rev))).last!
let suf = prefix_function(Str(chain(chain(mid.rev, "#"), mid))).last!
if pre >= suf: print(Str(chain(s[..l + pre], s[n - l..])))
else: print(Str(chain(s[..l], s[n - l - suf..])))
