@! multitest
let half: Int = read()
let n = 2 * half + 1
let s: Str = read()
var out: List<Char> = []

fn emit_expr(lo: Int, hi: Int):
    out.push(s[lo])
    var i = lo + 1
    while i < hi:
        out.push('?')
        out.push(s[i])
        out.push(':')
        out.push(s[i + 1])
        i += 2

var ok = true
if n >= 5 and s[n - 1] == '1' and s[0] == '1' and s[1] == '0':
    out.push('(')
    out.push(s[0])
    out.push('?')
    out.push(s[1])
    out.push(':')
    out.push(s[2])
    out.push(')')
    out.push('?')
    emit_expr(3, n - 1)
    out.push(':')
    out.push(s[n - 1])
elif s[n - 1] == '1' and s[0] == '0':
    out.push(s[0])
    out.push('?')
    emit_expr(1, n - 1)
    out.push(':')
    out.push(s[n - 1])
else:
    var i = 0
    while i < n and s[i] != '1':
        i += 2
    if i >= n:
        ok = false
    else:
        var j = i + 1
        while j < n and s[j] != '1':
            j += 2
        if j >= n:
            ok = false
        else:
            if i > 0:
                out.push(s[0])
                out.push('?')
                emit_expr(1, i)
                out.push(':')
            out.push(s[i])
            out.push('?')
            if j > i + 1:
                out.push('(')
                out.push(s[i + 1])
                out.push('?')
                emit_expr(i + 2, j)
                out.push(':')
                out.push(s[j])
                out.push(')')
            else:
                out.push(s[j])
            out.push(':')
            emit_expr(j + 1, n)
if ok:
    print("Yes")
    print(Str(out))
else:
    print("No")
