#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<long long> a(n), b(n);
    for (auto& x : a) if (scanf("%lld", &x) != 1) return 1;
    for (auto& x : b) if (scanf("%lld", &x) != 1) return 1;
    long long last = -1, bad = 0, sum = 0;
    auto emit = [&](long long x) { if (x < last) bad++; last = x; sum += x; };
    size_t i = 0, j = 0;
    while (i < a.size() && j < b.size()) {
        if (b[j] < a[i]) emit(b[j++]); else emit(a[i++]);
    }
    while (i < a.size()) emit(a[i++]);
    while (j < b.size()) emit(b[j++]);
    printf("%lld %lld\n", bad, sum);
}
