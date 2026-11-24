// IOI 2013 Dreaming is judged through a function, travelTime(n, m, l, a, b, t).
// This reads the same data from standard input (n m l, then m trails) so the
// accepted submission can be run beside the port.
#include "../../submissions/4687433.cpp"
#include <cstdio>

int main() {
    int n, m, l;
    if (scanf("%d %d %d", &n, &m, &l) != 3) return 1;
    std::vector<int> a(m + 1), b(m + 1), t(m + 1);
    for (int i = 0; i < m; i++) scanf("%d %d %d", &a[i], &b[i], &t[i]);
    printf("%d\n", travelTime(n, m, l, a.data(), b.data(), t.data()));
}
