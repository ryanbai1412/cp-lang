#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;

static constexpr uint32_t MOD = 998244353;

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int64_t n, seed;
    if (!(cin >> n >> seed)) return 0;
    vector<uint32_t> p(n + 1, 0);
    int64_t s = seed;
    for (int64_t i = n - 1; i >= 0; --i) {
        s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
        int64_t rem = n - 1 - i;
        uint32_t val = 1;
        if (rem > 0) {
            int64_t hi = i + 1 + (s & (rem - 1));
            uint32_t diff = p[i + 1] >= p[hi + 1] ? p[i + 1] - p[hi + 1] : p[i + 1] + MOD - p[hi + 1];
            val = diff + 1;
            if (val >= MOD) val -= MOD;
        }
        uint32_t nx = val + p[i + 1];
        if (nx >= MOD) nx -= MOD;
        p[i] = nx;
    }
    cout << p[0] << '\n';
}
