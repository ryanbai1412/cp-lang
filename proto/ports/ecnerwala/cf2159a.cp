@!interactive

fn ask(s: List<Int>) -> Int:
    print("?", s.len, s)
    read()

@!multitest
let n: Int = read()
var ans: [0..2 * n]Int = 0
var cur: List<Int> = []
var oth: List<Int> = []

for i in 0..2 * n:
    cur.push(i + 1)
    let res = ask(cur)
    if res != 0:
        ans[i] = res
        cur.pop()
        oth.push(i + 1)

for x in cur:
    oth.push(x)
    ans[x - 1] = ask(oth)
    oth.pop()

print("!", ans)
