#include <bits/stdc++.h>
using namespace std;

static inline int64_t step_fn(int64_t x, int64_t m) {
    if ((x & 3) == 0) return -1;
    return (x * 31 + 7) % m;
}

int64_t kernel(int n, int reps, const vector<int64_t>& a) {
    int64_t acc = 0;
    for (int r = 0; r < reps; ++r) {
        for (int i = 0; i < n; ++i) {
            int64_t v = step_fn(a[i] + r, 1000003);
            if (v != -1) {
                acc = (acc + v) ^ (acc >> 5);
            } else {
                acc = acc ^ (i + 1);
            }
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
