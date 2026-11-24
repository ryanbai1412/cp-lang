// expect: ok, output "3 5"
// rule: 10.2.3
// input: "2 4\n-1 3\n"
// read() on Range<D> and Range<Int> lowers and runs (reads two inclusive endpoints).
domain V = 1..=5
let dom: Range<V> = read()
let int: Range<Int> = read()
print(dom.len, int.len)
