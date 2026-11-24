#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<int> a(n), ord(n);
    for (auto& x : a) if (scanf("%d", &x) != 1) return 1;
    std::iota(ord.begin(), ord.end(), 0);
    std::sort(ord.begin(), ord.end(), [&](int i, int j) { return std::pair(a[i], i) < std::pair(a[j], j); });
    printf("%d %d\n", ord[0], ord[n - 1]);
}
