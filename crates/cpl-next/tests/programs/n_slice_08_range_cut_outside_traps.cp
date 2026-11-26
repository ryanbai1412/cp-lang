// expect: R0001
// rule: R07.2.15
// `(10..20)[15..25]` ends past the range it cuts.
print((10..20)[15..25].len)  // <- here
