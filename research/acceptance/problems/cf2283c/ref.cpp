#include <bits/stdc++.h>
using namespace std;
// m = (T ^ Y) ^ 2Y with T = xor(1..n), Y = xor of the doubled set; k = 2^L - 1.
// Pick m in {2^(L+1)-2, 2^(L+1)-1}, solve Y bit by bit, double Y's powers of two.
int main() {
    int t; scanf("%d", &t);
    string out;
    while (t--) {
        int n; scanf("%d", &n);
        int L = 32 - __builtin_clz(n), T = 0;
        for (int i = 1; i <= n; i++) T ^= i;
        int Y = 0;
        for (int low = 0; low < 2; low++) {
            int M = (1 << (L + 1)) - 2 + low, y = 0, prev = 0;
            for (int j = 0; j < L; j++) {
                int b = ((M >> j) ^ (T >> j) ^ prev) & 1;
                y |= b << j; prev = b;
            }
            if (prev == 1) { Y = y; break; }
        }
        vector<char> d(n + 1, 0);
        for (int j = 0; j < L; j++) if (Y >> j & 1) d[1 << j] = 1;
        for (int i = 1; i <= n; i++) { out += char('0' + d[i]); out += i == n ? '\n' : ' '; }
    }
    fwrite(out.data(), 1, out.size(), stdout);
}
