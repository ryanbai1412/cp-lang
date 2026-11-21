let n: Int = read()
domain P = 0..n
var ids: Map<Str, P> = Map<Str, P>()
var hs: HashMap<Int, Int> = HashMap<Int, Int>()
for i in 0..n:
    let name: Str = read()
    let p = ids.get_or_insert_with(name, () => P(ids.len))
    print(name, p)
    hs.get_or_insert_with(name.len, () => hs.len * 10)
print(ids.len, hs.len)
