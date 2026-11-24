let s: Str = read()
let w: Str = read()
let i = 0
let ch = s[i]
let sub = s[1..4]
var out = ""
out.push('a'); out += "bc" * 3
let line = read_line()
let (a, b) = "x y".split_once(' ')!
let parts = "he ll o".split(' ')
let joined = "-".join("{x}" for x in parts)
var t = s
t[0] = 'J'
print(ch, s[4] - 'a' - 10, sub, s[..2], s[2..], s.len, s.is_empty, out, s.starts_with("ll", at: 2), s, w, line.trim_start(' ').split(' ').first!.parse_int(), "{a},{b}", '7' - '0', t[0] - 'J' + 1)
print(joined)
