#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>

int main() {
    const int n = 200000;
    std::vector<int64_t> xs(n);
    int64_t seed = 123456789;
    for (int i = 0; i < n; i++) {
        seed = (seed * 1103515245LL + 12345LL) % 2147483647LL;
        xs[i] = seed;
    }

    int64_t acc = 0;
    for (int r = 0; r < 40; r++) {
        int base = (r * 3713) % (n / 2);
        int len = n / 2;
        int l = base;
        int hi = base + len;
        std::sort(xs.data() + l, xs.data() + hi);
        acc += xs[l] + xs[l + len / 2] + xs[hi - 1];
        for (int i = l; i < hi; i++) {
            seed = (seed * 1103515245LL + 12345LL) % 2147483647LL;
            xs[i] = xs[i] ^ seed;
        }
    }

    std::cout << acc << '\n';
    return 0;
}
