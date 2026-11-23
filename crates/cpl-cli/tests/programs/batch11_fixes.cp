// Batch #11: a range where a list is wanted, `reverse` on an array, `clear` on a `Str`.
let n: Int = read()
domain D = 0..n
let loot: [D]Int = read()
var best: [D]Int = 0
fn total(r: Range<D>) -> Int:
    sum(loot[d] for d in r)
fn count_all(xs: List<D>) -> Int: xs.len
for d in D:
    best[d] += loot[d]
print(total(D.all), best)
print(count_all(D.all))
var opt: [D]?Int = None
var m: [D]Mod<7> = 0
for d in D:
    m[d] += loot[d]
print(m)
let every: List<D> = D.all
var order: List<D> = D.all
order.reverse()
print(every.len, order[0], count_all(D.all))
fn span(xs: List<Int>) -> Int: xs.len
print(span(0..n), span(1..=n))
let k: Int = read()
domain I = 0..k
var a: [I]Int = read()
a.reverse()
print(a)
var b: [0..3]Int = 0
b[0] = 5
b.reverse()
print(b)
var s = "hello"
s.clear()
s.push('x')
print(s, s.len)
