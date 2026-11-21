// Open-addressing HashMap rehash safety: references into a table must not
// dangle when an assignment, compound op, key lookup, or nested mutation
// triggers a rehash of the same table (checked under ASan + UBSan).
var m: HashMap<Int, Int> = []
var ml: HashMap<Int, List<Int>> = []

fn grow() -> Int:
    for i in 10..40:
        m[i] = i
    5

fn grow_list() -> List<Int>:
    for i in 10..40:
        ml[i] = [i]
    [7, 8]

fn grow_int() -> Int:
    for i in 50..90:
        ml[i] = [i]
    9

fn grow_bool() -> Bool:
    for i in 100..150:
        ml[i] = [i]
    true

// Self-aliasing value copy across capacity-8 -> 16 rehash (List & Int).
ml[1] = [10, 20, 30]
ml[2] = [40, 50]
ml[3] = [60]
ml[4] = [70]
ml[5] = ml[1]
assert ml[5] == [10, 20, 30]

m[1] = 11
m[2] = 22
m[3] = 33
m[4] = 44
m[5] = m[1]
assert m[5] == 11

// Self-aliasing compound op, fold op, and key-from-self across rehash.
var m2: HashMap<Int, Int> = []
m2[1] = 100
m2[2] = 2
m2[3] = 3
m2[4] = 4
m2[5] += m2[1]
assert m2[5] == 100
m2[6] = 6
m2[7] = 7
m2[8] = 8
m2[m2[1]] = 99
assert m2[100] == 99

// Function-induced rehash during assignment, compound op, tuple swap, closure.
m[1] = grow()
assert m[1] == 5
m[2] += grow()
assert m[2] == 27
(m[3], m[99]) = (m[1], m[3])
assert m[3] == 5 and m[99] == 33
let read1 = () => m[1]
for i in 100..200: m[i] = i
assert read1() == 5

// Nested container mutations whose arguments rehash the outer HashMap.
ml[1] = [1, 2]
ml[1].extend(grow_list())
assert ml[1] == [1, 2, 7, 8]
ml[1].push(grow_int())
assert ml[1] == [1, 2, 7, 8, 9]
ml[1].sort(desc: grow_bool())
assert ml[1] == [9, 8, 7, 2, 1]
print("ok")
