// expect: ok
// rule: 10.2.3
// read() on Range<D> and Range<Int> lowers and runs (reads two inclusive endpoints).
domain V = 1..=5
fn read_dom() -> Range<V> = read()
fn read_int() -> Range<Int> = read()
