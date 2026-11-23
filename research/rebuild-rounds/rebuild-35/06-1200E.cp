// CF 1200E Compress Words: the overlap is the border of w + "#" + (tail of the result).
let n: 1..=1e5 = read()
let words = read<[n]Str>()
var res: Str = ""
for w in words:
    let k = min(w.len, res.len)
    let key = w + "#" + res[res.len - k..]
    let overlap = prefix_function(key)[key.len - 1]
    for c in w[overlap..]: res.push(c)
print(res)
