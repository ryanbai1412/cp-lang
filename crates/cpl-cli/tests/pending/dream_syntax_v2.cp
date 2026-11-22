fn cmp_check(a: Int, b: Int, by: fn(Int, Int) -> Bool) -> Bool = by(a, b)

print(yes(cmp_check(2, 5, by: <)), yes(cmp_check(5, 2, by: <)), yes(cmp_check(5, 2, by: >)))

let x: ?Int = some(10)
let y: ?Int = none
print(yes(x is v and v > 5), yes(y is v and v > 5))

if x is v and v == 10:
    print(v + 1)

var cur: ?Int = some(3)
while cur is n:
    print(n)
    cur = if n > 1: some(n - 1) else: none

enum Sign: neg = -1, zero = 0, pos = 1

let s0 = Sign.neg
let s1 = Sign.zero
let s2 = Sign.pos
print(Int(s0), Int(s1), Int(s2))
print(Int(Sign(-Int(s0))), Int(Sign(-Int(s2))))

struct Stair:
    lift: Int
    pts: List<Int>

var stairs: [0..2]Stair = [Stair(lift: 0, pts: []), Stair(lift: 10, pts: [])]
var k = 1
let st = mut stairs[k % 2]
k = 0
st.lift += 5
st.pts.push(42)
print(stairs[0].lift, stairs[1].lift, stairs[1].pts.len)

var nums: [0..3]Int = [1, 2, 3]
let elem = mut nums[1]
elem += 10
print(nums[0], nums[1], nums[2])
