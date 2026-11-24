#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<int> a(n);
    for (int& x : a) if (scanf("%d", &x) != 1) return 1;
    std::vector<std::pair<int, int>> s;
    s.reserve(n);
    for (int i = 0; i < n; i++) s.emplace_back(a[i], i);
    std::sort(s.begin(), s.end());
    printf("%d %d\n", s[0].second, s[n - 1].second);
}
