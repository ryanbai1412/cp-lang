// Runnable port (proto/cp.py lib-run) of mccc5j4.cp
// original: corpus/submissions/5333145.cpp

let k: Int = read()
var s: Str = read()

var rem = k
var wild = 0
for i in 0..s.len:
    let ch = s[i]
    if ch == '*':
        wild += 1
    else:
        rem -= ch - 'a' + 1

if rem < wild or rem > 26 * wild:
    print("Impossible")
else:
    for i in 0..s.len:
        if s[i] == '*':
            wild -= 1
            let d = max(1, rem - 26 * wild)
            s[i] = 'a' + (d - 1)
            rem -= d
    print(s)
