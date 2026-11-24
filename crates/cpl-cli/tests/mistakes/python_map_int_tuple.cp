// here: 3
// Python's `n, m = map(int, input().split())` (self-run mistakes): the fix was `(n, m) = (map(int, input().split()))`, then three more errors.
n, m = map(int, input().split())
print(n + m)
