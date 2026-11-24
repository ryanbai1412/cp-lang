// expect: E0501
// rule: 10.1.4
// A range with an open end has no C++ integer to cross to the judge's grader in.
@grader
fn total(a: List<0..>) -> Int = a.len // <- here
