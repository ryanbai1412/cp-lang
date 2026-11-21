#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int64_t n, q, seed;
    if (!(cin >> n >> q >> seed)) return 0;
    int64_t mask = n - 1;
    vector<int64_t> d(n + 1, 0);
    int64_t s = seed;
    for (int64_t k = 0; k < q; ++k) {
        s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
        int64_t x = s & mask;
        s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
        int64_t y = s & mask;
        s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
        int64_t v = (s & 255) - 128;
        int64_t l = min(x, y), r = max(x, y) + 1;
        d[l] += v;
        d[r] -= v;
    }
    vector<int64_t> out(n, 0);
    int64_t cur = 0;
    for (int64_t i = 0; i < n; ++i) {
        cur += d[i];
        out[i] = cur;
    }
    int64_t acc = 0;
    for (int64_t i = 0; i < n; ++i) acc += out[i];
    cout << acc << '\n';
}
