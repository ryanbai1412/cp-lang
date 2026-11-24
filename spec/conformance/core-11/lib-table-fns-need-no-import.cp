// expect: ok, output "1 10 YES 2 5 3"
// rule: 11.3.2
// Every free fn in a lib chapter's main table is a prelude name: no `use`.
print(cross(Pt(1, 0), Pt(0, 1)), choose(5, 2), yes(is_prime(97)), inv_mod(3, 5)!, suffix_array("abcab").len, floor_sum(3, 1, 1, 0))
