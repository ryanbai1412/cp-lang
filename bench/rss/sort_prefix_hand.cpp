#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<int> a(n), ord(n);
    for (auto& x : a) if (scanf("%d", &x) != 1) return 1;
    std::vector<long long> p(n + 1);
    for (int i = 0; i < n; i++) p[i + 1] = p[i] + a[i];
    std::iota(ord.begin(), ord.end(), 0);
    std::sort(ord.begin(), ord.end(), [&](int i, int j) { return std::pair(p[i], i) < std::pair(p[j], j); });
    printf("%d %d\n", ord[0], ord[n - 1]);
}
