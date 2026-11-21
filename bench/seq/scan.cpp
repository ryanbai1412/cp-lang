#include <algorithm>
#include <cstdint>
#include <iostream>
#include <numeric>
#include <vector>

int main() {
    const int n = 10000;
    std::vector<int64_t> a(n);
    int64_t seed = 123456789;
    for (int i = 0; i < n; i++) {
        seed = (seed * 1103515245LL + 12345LL) % 2147483647LL;
        a[i] = (seed % 2001) - 1000;
    }

    int64_t acc = 0;
    for (int r = 0; r < 2000; r++) {
        std::vector<int64_t> ps(n);
        std::partial_sum(a.begin(), a.end(), ps.begin());
        std::vector<int64_t> pm(n);
        std::partial_sum(a.begin(), a.end(), pm.begin(),
                         [](int64_t x, int64_t y) { return std::min(x, y); });
        std::vector<int64_t> sm(n);
        std::partial_sum(a.rbegin(), a.rend(), sm.rbegin(),
                         [](int64_t x, int64_t y) { return std::max(x, y); });
        acc += ps[n - 1 - (r % n)] + pm[n - 1 - (r % n)] + sm[r % n];
    }

    std::cout << acc << '\n';
    return 0;
}
