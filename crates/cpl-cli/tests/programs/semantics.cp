// The corners where C++ and the language could part ways: the order
// things run in, copies, names C++ reserves, and shadowing.
var trace: List<Int> = []

fn tick(x: Int) -> Int:
    trace.push(x)
    x

fn pair(a: Int, b: Int) -> Int:
    a * 10 + b

// Operands, arguments, indices and elements run left to right; the
// indices of an assignment run before its right side (v2.2).
print(tick(1) - tick(2), pair(tick(3), tick(4)), [tick(5), tick(6)], (tick(7), tick(8)))
var grid: [3, 3]Int = 0
grid[tick(1), tick(2)] = tick(9)
print(grid[1, 2], trace)
trace.clear()
print("{tick(1)} then {tick(2)}", tick(3), trace.len)
var xs: List<Int> = [10, 20, 30]
xs[tick(0)] += tick(5)
xs[tick(1)] max= tick(99)
print(xs, trace)

// Assignment copies: arrays, lists and structs are values.
struct Box:
    var items: List<Int>
    var tag: Int

var a = Box(items: [1, 2], tag: 1)
var b = Box(items: a.items.copy(), tag: a.tag)
b.items.push(3)
b.tag = 2
print(a, "|", b)
var row: [3]Int = 7
var other = row.copy()
other[0] = 0
print(row, other)
var lists: List<List<Int>> = [[1], [2]]
var first = lists[0]
first.push(100)
lists[1].push(200)
print(lists, first)

// Names that mean something to C++.
let int = 1
let double = 2
let class = 3
let new = 4
let std = 5
let ll = 6
let main = 7
let solve = 8
let cpl = 9
let auto = 10
let t0 = 11
print(int + double + class + new + std + ll + main + solve + cpl + auto + t0)

fn delete(namespace: Int) -> Int:
    let template = namespace + 1
    template

print(delete(41))

// Shadowing: each `let` is a new variable.
let v = 1
if v == 1:
    let v = "inner"
    print(v)
print(v)
for v in 0..2:
    let v = v * 100
    print(v)
var total = 0
for i in 0..3:
    var i = i
    i += 10
    total += i
print(total)

// `and` and `or` stop early.
fn loud(b: Bool, x: Int) -> Bool:
    trace.push(x)
    b

trace.clear()
print(yes(loud(false, 1) and loud(true, 2)), yes(loud(true, 3) or loud(false, 4)), trace)
print(yes(not true or true and false), 1 + 2 * 3 - 4 / 2, -2 ** 2, yes((1 < 2) == (3 > 2)), yes(1 < 2 < 3))
print(7 - 3 - 2, 100 / 10 / 5, 2 ** 3 ** 2, 5 - -3, - - 4, 10 - (4 - 3))
// A value block as an expression.
let big = do:
    var s = 0
    for i in 1..=10:
        s += i
    s
print(big)
let t = if big > 50: big - 50 else: 0
print(t)
