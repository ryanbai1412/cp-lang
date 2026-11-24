// expect: ok
// rule: 10.1.4
// tool: build {case} -o -
// has: "std::pair<std::vector<int>, long long> transaction(long long m);"
// has: "void buy_souvenirs(int count, long long p0)"
// A value crossing to the grader takes the narrowest of `int` and `long long` that holds its declared range, and a pair is `std::pair`.
@grader fn transaction(m: 1..=1e15) -> (List<0..100>, 0..=1e15)
@grader
fn buy_souvenirs(count: 2..=100, p0: 1..=1e15):
    let (got, back) = transaction(p0 - 1)
    print(got.len, back, count)
