// Batch #14: a `let` that names a list where it lives, and a search of
// a field, read in place; both back ends must still agree.
let n: Int = read()
var rows: List<List<Int>> = [[i * j for j in 0..=i] for i in 0..n]

fn last_of(r: Int) -> Int:
    let row = rows[r]
    row[row.len - 1]

fn total(table: List<List<Int>>, r: Int) -> Int:
    let row = table[r]
    var s = 0
    for x in row: s += x
    s

// The fn changes `rows`, so `row` must be a copy that stays as it was.
fn grow(r: Int) -> Int:
    let row = rows[r].copy()
    rows[r].push(100)
    rows.push([1, 2, 3])
    row.len

print(last_of(2), total(rows, 3))
print(grow(1), rows[1].len, rows.len)

fn fmt_i(x: ?Int) -> Str = if let v = x: "{v}" else: "none"
let (domain Vc, vals) = compress([30, 10, 20, 10])
print(vals.at(20).ord, if let v = vals.first_ge(15): "{v.ord}" else: "none", if let v = vals.first_ge(31): "{v.ord}" else: "none")

struct Bag:
    items: List<Int>
    fn where_is(self, x: Int) -> ?Int = items.find(x)
    fn from(self, x: Int) -> ?Int = items.first_ge(x)

let bag = Bag(items: [2, 4, 6])
print(fmt_i(bag.where_is(4)), fmt_i(bag.where_is(5)), fmt_i(bag.from(5)))
