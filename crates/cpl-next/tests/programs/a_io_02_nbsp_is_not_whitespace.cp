// expect: R0501
// rule: R10.2.8 lists the whitespace characters
// input: "1\u00a02\n"
// The interpreter splits on Unicode whitespace, the C++ on C whitespace.
let (a, b): (Int, Int) = read()  // <- here
print(a, b)
