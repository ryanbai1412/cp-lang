// UCup 4 Stage 25 (Zagreb) H: order single-character deletions lexicographically
// by scanning right-to-left transitions, then erase the median index N / 2.
@!multitest
let s: Str = read()
let n = s.len
var left: List<Int> = []
var right: List<Int> = []
var state = false
for i in (0..n).rev:
    if state: right.push(i)
    else: left.push(i)
    if i > 0 and s[i] != s[i - 1]:
        state = s[i - 1] > s[i]
left.reverse()
for idx in right: left.push(idx)
let del = left[n / 2]
print(s[0..del] + s[del + 1..n])
