// expect: E0401
// rule: 05.3.1
// A `mut self` method needs a place to keep its change; on a temporary the error names the method, not an assignment.
let xs = [3, 1, 3]
let cool = [x * 2 for x in xs].sorted().dedup() // <- here
