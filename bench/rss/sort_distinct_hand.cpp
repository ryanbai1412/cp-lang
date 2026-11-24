#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<int> a(n), ord(n);
    for (auto& x : a) if (scanf("%d", &x) != 1) return 1;
    std::iota(ord.begin(), ord.end(), 0);
    std::sort(ord.begin(), ord.end(), [&](int i, int j) { return a[i] % 1000 < a[j] % 1000 || (a[i] % 1000 == a[j] % 1000 && i < j); });
    long long s = 0;
    for (int k = 0; k < n; k++) s += (long long)ord[k] * (k % 7);
    printf("%lld\n", s);
}
