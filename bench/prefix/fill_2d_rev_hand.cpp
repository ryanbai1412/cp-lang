#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;

static constexpr uint32_t MOD = 998244353;

static inline uint32_t addm(uint32_t a, uint32_t b) {
    uint32_t s = a + b;
    return s >= MOD ? s - MOD : s;
}
static inline uint32_t subm(uint32_t a, uint32_t b) {
    return a >= b ? a - b : a + MOD - b;
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int64_t h, w, seed;
    if (!(cin >> h >> w >> seed)) return 0;
    int64_t W = w + 1;
    vector<uint32_t> p((h + 1) * W, 0);
    int64_t s = seed;
    for (int64_t r = h - 1; r >= 0; --r) {
        for (int64_t c = w - 1; c >= 0; --c) {
            s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
            int64_t rr = h - 1 - r;
            int64_t rc = w - 1 - c;
            uint32_t val = 1;
            if (rr > 0 && rc > 0) {
                int64_t r1 = r + 1 + (s & (rr - 1));
                s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
                int64_t c1 = c + 1 + (s & (rc - 1));
                val = addm(addm(subm(subm(p[(r + 1) * W + (c + 1)], p[(r1 + 1) * W + (c + 1)]), p[(r + 1) * W + (c1 + 1)]), p[(r1 + 1) * W + (c1 + 1)]), 1);
            }
            p[r * W + c] = subm(addm(addm(val, p[(r + 1) * W + c]), p[r * W + (c + 1)]), p[(r + 1) * W + (c + 1)]);
        }
    }
    cout << p[0] << '\n';
}
