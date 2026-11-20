// section 8: a zero-arg `@memo fn` is a once-cell (the body runs once)
@memo
fn base() -> Int: 
    print("once")
    return 7
print(base() + base())
