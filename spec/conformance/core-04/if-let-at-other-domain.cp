// expect: E0302
// rule: 04.2.14
// The pattern `V(j)` matches an `Int`; a position of another domain is not one.
domain V = 0..5
domain W = 0..5
let w = W(1)
if let V(j) = w: // <- here
    print(j)
