// expect: ok, output "0 4 -1 0"
// rule: 09.2.3
// A predicate true everywhere or nowhere is monotone both ways, and an empty or one-element search has no other end to check, so none of these trap.
let st = [1, 3, 5, 7]
print(st.bsearch(t => t < 100).r, st.bsearch(t => t > 100).r, (0..0).first_true(x => x < 4) ?? -1, [5].bsearch(t => t < 9).r)
