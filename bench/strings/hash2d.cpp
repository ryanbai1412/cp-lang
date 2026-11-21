#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;

static constexpr int64_t M1 = 1000000007;
static constexpr int64_t M2 = 1000000009;

static inline int64_t modp(int64_t a, int64_t m) {
    int64_t r = a % m;
    return r < 0 ? r + m : r;
}

int main() {
    const int rows = 500, cols = 500;
    vector<int64_t> grid(rows * cols);
    int64_t x = 1234567;
    for (int i = 0; i < rows; i++) {
        for (int j = 0; j < cols; j++) {
            x = (x * 1103515245 + 12345) & 0x7fffffff;
            grid[i * cols + j] = (x >> 16) % 26 + 97;
        }
    }
    int64_t br1 = 131, bc1 = 137;
    int64_t br2 = (br1 ^ 324508639) % (M2 - 2) + 2;
    int64_t bc2 = (bc1 ^ 715827883) % (M2 - 2) + 2;
    int stride = cols + 1;
    vector<int64_t> pre1((rows + 1) * stride, 0), pre2((rows + 1) * stride, 0);
    for (int i = 0; i < rows; i++) {
        for (int j = 0; j < cols; j++) {
            int64_t v = grid[i * cols + j];
            int64_t v1 = v % (M1 - 1) + 1;
            int64_t v2 = v % (M2 - 1) + 1;
            int64_t a1 = (pre1[i * stride + j + 1] * br1 + pre1[(i + 1) * stride + j] * bc1) % M1;
            int64_t b1 = pre1[i * stride + j] * br1 % M1 * bc1 % M1;
            pre1[(i + 1) * stride + j + 1] = modp(a1 - b1 + v1, M1);
            int64_t a2 = (pre2[i * stride + j + 1] * br2 + pre2[(i + 1) * stride + j] * bc2) % M2;
            int64_t b2 = pre2[i * stride + j] * br2 % M2 * bc2 % M2;
            pre2[(i + 1) * stride + j + 1] = modp(a2 - b2 + v2, M2);
        }
    }
    vector<int64_t> pow_r1(rows + 1, 1), pow_r2(rows + 1, 1);
    for (int i = 0; i < rows; i++) {
        pow_r1[i + 1] = pow_r1[i] * br1 % M1;
        pow_r2[i + 1] = pow_r2[i] * br2 % M2;
    }
    vector<int64_t> pow_c1(cols + 1, 1), pow_c2(cols + 1, 1);
    for (int j = 0; j < cols; j++) {
        pow_c1[j + 1] = pow_c1[j] * bc1 % M1;
        pow_c2[j + 1] = pow_c2[j] * bc2 % M2;
    }

    int64_t acc = 0;
    for (int q = 0; q < 500000; q++) {
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        int r1 = int((x >> 8) % rows);
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        int r2 = int((x >> 8) % rows);
        int ra = min(r1, r2), rb = max(r1, r2) + 1;
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        int c1 = int((x >> 8) % cols);
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        int c2 = int((x >> 8) % cols);
        int ca = min(c1, c2), cb = max(c1, c2) + 1;
        int64_t hr1 = pow_r1[rb - ra], hc1 = pow_c1[cb - ca];
        int64_t h1 = modp(pre1[rb * stride + cb] - pre1[ra * stride + cb] * hr1 -
                              pre1[rb * stride + ca] * hc1 +
                              pre1[ra * stride + ca] * hr1 % M1 * hc1,
                          M1);
        int64_t hr2 = pow_r2[rb - ra], hc2 = pow_c2[cb - ca];
        int64_t h2 = modp(pre2[rb * stride + cb] - pre2[ra * stride + cb] * hr2 -
                              pre2[rb * stride + ca] * hc2 +
                              pre2[ra * stride + ca] * hr2 % M2 * hc2,
                          M2);
        acc ^= h1 ^ (h2 << 1);
    }
    cout << acc << '\n';
}
