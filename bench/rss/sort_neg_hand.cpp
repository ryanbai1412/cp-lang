#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<int> a(n), ord(n), down(n);
    for (auto& x : a) if (scanf("%d", &x) != 1) return 1;
    std::iota(ord.begin(), ord.end(), 0);
    std::sort(ord.begin(), ord.end(), [&](int i, int j) { return std::pair(-a[i], i) < std::pair(-a[j], j); });
    std::iota(down.begin(), down.end(), 0);
    std::sort(down.begin(), down.end(), [&](int i, int j) { return a[i] > a[j] || (a[i] == a[j] && i < j); });
    printf("%d %d %d %d\n", ord[0], ord[n - 1], down[0], down[n - 1]);
}
