// Matched C++ for matlazy.cp: each row sums `a * b` unreduced, reducing every 8 steps.
#include <cstdio>
#include <vector>
using ll = long long;
const ll P = 998244353;
const int n = 64;
using mat = std::vector<ll>;
mat mul(const mat& a, const mat& b) {
    mat r(n * n);
    std::vector<ll> row(n);
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) row[j] = 0;
        for (int k = 0; k < n; k++) {
            if (k % 8 == 0)
                for (int j = 0; j < n; j++) row[j] %= P;
            ll f = a[i * n + k];
            for (int j = 0; j < n; j++) row[j] += f * b[k * n + j];
        }
        for (int j = 0; j < n; j++) r[i * n + j] = row[j] % P;
    }
    return r;
}
int main() {
    ll s = 7;
    mat d(n * n);
    for (int i = 0; i < n * n; i++) {
        s = (s * 1103515245 + 12345) & 2147483647;
        d[i] = s % P;
    }
    mat m = d;
    for (int t = 0; t < 400; t++) m = mul(m, d);
    ll acc = 0;
    for (int i = 0; i < n * n; i++) acc = (acc + m[i]) % P;
    printf("%lld\n", acc);
}
