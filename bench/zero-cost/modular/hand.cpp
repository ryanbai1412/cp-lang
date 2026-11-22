#include <bits/stdc++.h>
using namespace std;

int64_t kernel(int n, int reps, const vector<int64_t>& a) {
    constexpr int64_t P = 998244353;
    int64_t acc = 1;
    for (int r = 0; r < reps; ++r) {
        int64_t shift = (r + 1) % P;
        for (int i = 0; i < n; ++i) {
            int64_t v = (acc * 31) % P + (a[i] % P);
            if (v >= P) v -= P;
            v += shift;
            if (v >= P) v -= P;
            acc = v;
        }
    }
    return acc;
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n, reps;
    if (!(cin >> n >> reps)) return 0;
    vector<int64_t> a(n);
    for (int i = 0; i < n; ++i) cin >> a[i];
    cout << kernel(n, reps, a) << '\n';
}
