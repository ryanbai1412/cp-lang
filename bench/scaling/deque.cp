// let: k = n / 4
use std.deque.{LinkedList}
let n: Int = read()
let xs = [i for i in 0..n]
var s = 0
let front = (x: Int) => x < n / 4 // within: deque.Deque.pop_front_while
let back = (x: Int) => x >= n - n / 4 // within: deque.Deque.pop_while
for r in 0..128:
    var q = Deque<Int>.of(xs)
    s += q.pop_front_while(front).len // timed: deque.Deque.pop_front_while
    s += q.pop_while(back).len // timed: deque.Deque.pop_while
    var ll = LinkedList<Int>.from_list(xs) // timed: deque.LinkedList.from_list
    s += ll.len
for r in 0..512:
    var q = Deque<Int>.of(xs) // timed: deque.Deque.of
    q.rotate_left(n / 4) // timed: deque.Deque.rotate_left
    q.rotate_left(n / 4) // timed: deque.Deque.rotate_left
    q.rotate_right(n / 4) // timed: deque.Deque.rotate_right
    q.rotate_right(n / 4) // timed: deque.Deque.rotate_right
    s += q.to_list().len // timed: deque.Deque.to_list
    var f = Deque<Int>.of([r])
    f.reserve(n) // timed: deque.Deque.reserve
    var f2 = Deque<Int>.of([r])
    f2.reserve(n) // timed: deque.Deque.reserve
var d = Deque<Int>.of(xs)
for i in 0..8 * n:
    d.push_front(i) // timed: deque.Deque.push_front
    s += d.pop_front() ?? 0 // timed: deque.Deque.pop_front
    d.push(i)
    s += d.pop() ?? 0 // timed: deque.Deque.pop
    s += d.first ?? 0 // timed: deque.Deque.first
    s += d.last ?? 0 // timed: deque.Deque.last
    s += d.get(i % n) // timed: deque.Deque.get
    d.set(i % n, i) // timed: deque.Deque.set
    s += d.len // timed: deque.Deque.len
    if d.is_empty: s += 1 // timed: deque.Deque.is_empty
    var z = Deque<Int>() // timed: deque.Deque.new
    z.push(i)
    var t = z.take() // timed: deque.Deque.take
    t.clear() // timed: deque.Deque.clear
    s += t.len + z.len
var l = LinkedList<Int>.from_list(xs)
for i in 0..8 * n:
    var a = LinkedList<Int>() // timed: deque.LinkedList.new
    a.push(i) // timed: deque.LinkedList.push
    a.push_front(i) // timed: deque.LinkedList.push_front
    a.splice_back(LinkedList<Int>.of(i)) // timed: deque.LinkedList.splice_back
    a.splice_front(LinkedList<Int>.of(i)) // timed: deque.LinkedList.splice_front
    var b = LinkedList<Int>.of(i) // timed: deque.LinkedList.of
    a.reverse() // timed: deque.LinkedList.reverse
    s += a.first ?? 0 // timed: deque.LinkedList.first
    s += a.last ?? 0 // timed: deque.LinkedList.last
    s += a.len // timed: deque.LinkedList.len
    if a.is_empty: s += 1 // timed: deque.LinkedList.is_empty
    s += a.pop() ?? 0 // timed: deque.LinkedList.pop
    s += a.pop_front() ?? 0 // timed: deque.LinkedList.pop_front
    a.clear() // timed: deque.LinkedList.clear
    s += b.len
for _ in 0..16:
    for (i, x) in d.items: // timed: deque.Deque.items
        s += i ^ x // timed: deque.Deque.items
    for x in d.each(): // timed: deque.Deque.each
        s += x // timed: deque.Deque.each
    for x in d.each_rev(): // timed: deque.Deque.each_rev
        s ^= x // timed: deque.Deque.each_rev
    for x in l.each(): // timed: deque.LinkedList.each
        s += x // timed: deque.LinkedList.each
    for x in l.each_rev(): // timed: deque.LinkedList.each_rev
        s ^= x // timed: deque.LinkedList.each_rev
print(s)
