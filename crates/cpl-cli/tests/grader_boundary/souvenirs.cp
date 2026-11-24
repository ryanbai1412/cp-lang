// The grader's `transaction` and `buy_souvenirs`, in their declared types.
@grader fn transaction(m: 1..=1e15) -> (List<0..100>, 0..=1e15)

@grader
fn buy_souvenirs(n: 2..=100, p0: 1..=1e15):
    let (got, back) = transaction(p0 - 1)
    print(got.len, got[0], back, n)
