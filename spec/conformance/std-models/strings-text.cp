// expect: ok, output "ok"
// input: "1 300\n"
// model: lower
// model: upper
// model: digit
// model: is_digit
// model: is_lower
// model: is_upper
// model: is_alpha
// model: is_alnum
// model: is_space
// model: to_lower
// model: to_upper
// model: trim
// model: trim_start
// model: trim_end
// model: split_once
// model: split_whitespace
// model: join
// model: replace
// model: repeat
// model: parse_int
// model: parse
// model: starts_with
// model: ends_with
// model: search
// model: rsearch
// model: common_prefix_len
// model: common_suffix_len
// The text helpers on short random strings over a mixed alphabet (letters, digits, signs, blanks),
// each against a loop over the characters. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}

fn at(s: Str, i: Int, p: Str) -> Bool = i + p.len <= s.len and (0..p.len).all(j => s[i + j] == p[j])
fn cut(s: Str, lo: Int, hi: Int) -> Str = Str([s[j] for j in lo..hi])

fn digits_of(s: Str, base: Int) -> ?Int:
    let neg = s.len > 0 and s[0] == '-'
    let lo = if neg: 1 else: 0
    if lo == s.len: return None
    var x = 0
    for i in lo..s.len:
        let c = Int(s[i])
        let d = if 48 <= c <= 57: c - 48 elif 97 <= c <= 122: c - 87 elif 65 <= c <= 90: c - 55 else: 99
        if d >= base or x > (Int.MAX - d) / base: return None
        x = x * base + d
    Some(if neg: -x else: x)

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
let alpha = "aAbBzZ09 \t\n-+=x"
var bad = ""
for k in 0..n:
    let s = Str([alpha[rng.int(0, alpha.len - 1)] for _ in 0..rng.int(0, 10)])
    let p = Str([alpha[rng.int(0, 5)] for _ in 0..rng.int(0, 2)])
    for c in s:
        let o = Int(c)
        let low = 97 <= o <= 122
        let up = 65 <= o <= 90
        let dig = 48 <= o <= 57
        if c.is_lower != low or c.is_upper != up or c.is_digit != dig or c.is_alpha != (low or up) or c.is_alnum != (low or up or dig):
            bad = "op {k}: class of {o}"
        elif c.is_space != (o == 32 or o == 9 or o == 10 or o == 13): bad = "op {k}: is_space {o}"
        elif Int(c.lower) != (if up: o + 32 else: o) or Int(c.upper) != (if low: o - 32 else: o): bad = "op {k}: case of {o}"
        elif dig and c.digit != o - 48: bad = "op {k}: digit {o}"
    if bad != "": break
    if s.to_lower() != Str([c.lower for c in s]) or s.to_upper() != Str([c.upper for c in s]): bad = "op {k}: to_lower/to_upper"
    var a = 0
    while a < s.len and s[a].is_space: a += 1
    var b = s.len
    while b > a and s[b - 1].is_space: b -= 1
    var e = s.len
    while e > 0 and s[e - 1].is_space: e -= 1
    if s.trim_start() != cut(s, a, s.len) or s.trim_end() != cut(s, 0, e) or s.trim() != cut(s, a, b): bad = "op {k}: trim"
    if bad != "": break
    var once: ?(Str, Str) = None
    for i in 0..s.len:
        if s[i] == '=' and once == None: once = Some((cut(s, 0, i), cut(s, i + 1, s.len)))
    var words: List<Str> = []
    var i = 0
    while i < s.len:
        if s[i].is_space: i += 1
        else:
            var j = i
            while j < s.len and not s[j].is_space: j += 1
            words.push(cut(s, i, j))
            i = j
    if s.split_once('=') != once: bad = "op {k}: split_once"
    elif s.split_whitespace() != words: bad = "op {k}: split_whitespace"
    elif "-".join(words) != (if words.len == 0: "" else: words[0] + "".join(["-" + w for w in words[1..]])): bad = "op {k}: join"
    if bad != "": break
    let r = rng.int(0, 3)
    var rep = ""
    for _ in 0..r: rep += s
    if s.repeat(r) != rep: bad = "op {k}: repeat {r}"
    if p.len > 0:
        var out = ""
        var i = 0
        while i < s.len:
            if at(s, i, p):
                out += "#"
                i += p.len
            else:
                out += "{s[i]}"
                i += 1
        if s.replace(p, "#") != out: bad = "op {k}: replace"
    if bad != "": break
    let num = Str([("-+0123456789aZ "[rng.int(0, 14)]) for _ in 0..rng.int(0, 4)]) + Str(["0123456789"[rng.int(0, 9)] for _ in 0..rng.int(0, 21)])
    let base = [10, 16, 36, 2][rng.int(0, 3)]
    if num.parse_int(base: base) != digits_of(num, base): bad = "op {k}: parse_int({num}, base: {base})"
    elif num.parse<Int>() != (if num.split_whitespace().len == 1: digits_of(num.trim(), 10) else: None) and "+" not in num and num.len <= 18: bad = "op {k}: parse<Int>({num})"
    if bad != "": break
    let off = rng.int(0, s.len)
    var first: ?Int = None
    var last: ?Int = None
    for i in 0..s.len:
        if at(s, i, p):
            if first == None: first = Some(i)
            last = Some(i)
    var cp = 0
    while cp < s.len and cp < p.len and s[cp] == p[cp]: cp += 1
    var cs = 0
    while cs < s.len and cs < p.len and s[s.len - 1 - cs] == p[p.len - 1 - cs]: cs += 1
    if s.starts_with(p) != at(s, 0, p) or s.starts_with(p, at: off) != at(s, off, p): bad = "op {k}: starts_with at {off}"
    elif s.ends_with(p) != (p.len <= s.len and at(s, s.len - p.len, p)): bad = "op {k}: ends_with"
    elif s.search(p) != first or s.rsearch(p) != last: bad = "op {k}: search/rsearch"
    elif s.common_prefix_len(p) != cp or s.common_suffix_len(p) != cs: bad = "op {k}: common_prefix/suffix_len"
    if bad != "": break
print(if bad == "": "ok" else: bad)
