let n: Int = read()
domain P = 0..n
var ids: Map<Str, P> = Map<Str, P>()
var hs: HashMap<Int, Int> = HashMap<Int, Int>()
for i in 0..n:
    let name: Str = read()
    let p = ids.get_or_insert(name, P(ids.len))
    print(name, p)
    hs.update(name.len, -1, v => if v < 0: hs.len * 10 else: v)
print(ids.len, hs.len)
