struct N:
    k: Int
    l: ?N
struct A:
    b: ?(Int, B)
struct B:
    a: A
enum T:
    Leaf
    Node(l: T, r: List<T>)
struct Ok:
    kids: List<Ok>
print(N(1, None).k)
