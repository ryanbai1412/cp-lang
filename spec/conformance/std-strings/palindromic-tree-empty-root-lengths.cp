// expect: ok, output "-1 0"
use std.strings.{PalindromicTree}
let t = PalindromicTree<Char>.of("")
print(t.pal_len(0), t.pal_len(1))
