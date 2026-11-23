# Exact reference from the statement: count right triangles containing each point, borders included.
import sys
def main():
    it = iter(sys.stdin.read().split()); nx = lambda: int(next(it))
    n, q = nx(), nx()
    tris = [tuple(nx() for _ in range(6)) for _ in range(n)]
    def cross(o, a, b): return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    out = []
    for _ in range(q):
        p = (nx(), nx()); k = 0
        for x1, y1, x2, y2, x3, y3 in tris:
            a, b, c = (x1, y1), (x2, y2), (x3, y3)
            d = (cross(a, b, p), cross(b, c, p), cross(c, a, p))
            if not (min(d) < 0 < max(d)): k += 1
        out.append(k)
    print("\n".join(map(str, out)))
main()
