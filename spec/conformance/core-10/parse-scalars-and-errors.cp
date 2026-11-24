// expect: ok, output "-12 3 2.5000000000 4 5 YES YES YES YES"
// rule: 10.2.4
let (a, b) = "4 5".parse<(Int, Int)>()!
print("-12".parse<Int>()!, "3".parse<1..=3>()!, "2.5".parse<F64>()!, a, b,
    yes("x".parse<Int>() == None), yes("4".parse<1..=3>() == None),
    yes("2.5.1".parse<F64>() == None), yes("4".parse<(Int, Int)>() == None))
