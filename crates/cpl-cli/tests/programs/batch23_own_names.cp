// A program's own fn with the name of a library method is the one that
// runs when it is called by name, `f(x, ..)`, and for a method the library
// writes in `.cp` (`get_or_insert`) also as `x.f(..)`: no back end swaps in
// its own. Each of these does something the library's does not.

// Overwrites what is there, and gives back the old value.
fn get_or_insert(mut self: Map<Int, Int>, k: Int, v: Int) -> Int:
    let old = self.get(k) ?? -1
    self[k] = v
    old

// Counts the values that are NOT x.
fn count(self: List<Int>, x: Int) -> Int:
    var others = 0
    for y in self:
        if y != x: others += 1
    others

// Reverses, and says how many cells it moved.
fn sort(mut self: List<Int>) -> Int:
    self.reverse()
    self.len

// Leaves the set alone.
fn remove(self: Set<Int>, x: Int) -> Bool = x in self

var m = Map<Int, Int>()
print(m.get_or_insert(1, 5), m.get_or_insert(1, 9), m[1], m.len)
print(get_or_insert(mut m, 1, 2), m[1])

var xs = [3, 1, 2, 1]
print(count(xs, 3), count(xs, 1))
print(sort(mut xs), xs)
print(sort(mut xs), xs)

var s = Set<Int>()
s.insert(4)
print(remove(s, 4), remove(s, 5), s.len)
