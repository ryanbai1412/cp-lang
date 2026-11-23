// expect: ok, output "3"
// rule: 07.5.10
// A view of an unnamed temporary may be used to the end of its statement (R07.5.10).
print([3, 1, 2].sorted()[0..2].sum())
