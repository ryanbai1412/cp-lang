#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<int> a(n);
    for (auto& x : a) if (scanf("%d", &x) != 1) return 1;
    std::nth_element(a.begin(), a.begin() + 29, a.end(), std::greater<int>());
    std::sort(a.begin(), a.begin() + 30);
    int h0 = a[0], h29 = a[29];
    std::nth_element(a.begin(), a.begin() + 29, a.end());
    std::sort(a.begin(), a.begin() + 30);
    printf("%d %d %d %d\n", h0, h29, a[0], a[29]);
}
