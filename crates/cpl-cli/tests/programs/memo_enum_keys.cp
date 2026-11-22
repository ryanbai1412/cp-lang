// `@memo` keyed by a plain enum and by an optional enum: a dense table,
// with `none` as one more row.
enum Row = Top | Bottom

fn other(r: Row) -> Row = if r == Row.Top: Row.Bottom else: Row.Top

@memo
fn paths(c: 0..=40, r: Row) -> Int:
    if c == 0: 1
    else: paths(c - 1, r) + paths(c - 1, other(r))

@memo
fn best(c: 0..=40, behind: ?Row) -> Int:
    if c == 0: 0
    elif let z = behind: best(c - 1, none) + (if z == Row.Top: 2 else: 1)
    else: max(best(c - 1, some(Row.Top)), best(c - 1, some(Row.Bottom)))

print(paths(40, Row.Top), paths(3, Row.Bottom))
print(best(40, none), best(39, some(Row.Bottom)))

enum Dir: N = 4, S = 9, E = 1

@memo
fn doubling(d: Dir, k: 0..=5) -> Int:
    if k == 0: 1 else: doubling(d, k - 1) * 2

@memo
fn shifted(x: ?0..=10) -> Int:
    if let v = x: v + 1 else: 0

print(doubling(Dir.S, 5), doubling(Dir.E, 0), shifted(some(10)), shifted(none), shifted(some(0)))
