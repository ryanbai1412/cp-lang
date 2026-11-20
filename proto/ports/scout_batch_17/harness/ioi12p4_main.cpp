// IOI 2012 Ideal City is judged through a function, DistanceSum(n, x, y).
// This reads the same data from standard input (n, then n blocks) so the
// accepted submission can be run beside the port.
#include "../../../../corpus/submissions/4951048.cpp"
#include <cstdio>

int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<int> x(n), y(n);
    for (int i = 0; i < n; i++) scanf("%d %d", &x[i], &y[i]);
    printf("%d\n", DistanceSum(n, x.data(), y.data()));
}
