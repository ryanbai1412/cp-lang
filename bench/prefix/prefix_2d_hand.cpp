#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int64_t h, w, q, seed;
    if (!(cin >> h >> w >> q >> seed)) return 0;
    int64_t rmask = h - 1, cmask = w - 1;
    int64_t s = seed;
    vector<int64_t> a(h * w);
    for (int64_t r = 0; r < h; ++r) {
        for (int64_t c = 0; c < w; ++c) {
            s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
            a[r * w + c] = (s & 255) - 128;
        }
    }
    int64_t W = w + 1;
    vector<int64_t> p((h + 1) * W, 0);
    for (int64_t r = 0; r < h; ++r) {
        for (int64_t c = 0; c < w; ++c) {
            p[(r + 1) * W + (c + 1)] =
                p[r * W + (c + 1)] +
                p[(r + 1) * W + c] -
                p[r * W + c] +
                a[r * w + c];
        }
    }
    int64_t acc = 0;
    for (int64_t k = 0; k < q; ++k) {
        s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
        int64_t r1 = s & rmask;
        s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
        int64_t r2 = s & rmask;
        s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
        int64_t c1 = s & cmask;
        s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
        int64_t c2 = s & cmask;
        int64_t ra = min(r1, r2), rb = max(r1, r2) + 1;
        int64_t ca = min(c1, c2), cb = max(c1, c2) + 1;
        acc += p[rb * W + cb] - p[ra * W + cb] -
               p[rb * W + ca] + p[ra * W + ca];
    }
    cout << acc << '\n';
}
