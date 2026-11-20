#pragma GCC optimize("Ofast")
#pragma GCC target("avx2")

#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

vector<int> distribute_candies(vector<int> _arr,
                               vector<int> _ql,
                               vector<int> _qr,
                               vector<int> _qv) {
    int n = sz(_arr), q = sz(_ql), *arr = _arr.data(), *ql = _ql.data(),
        *qr = _qr.data(), *qv = _qv.data();
    int ans[n];
    for (int qi = 0; qi < q; qi++) {
        int l = ql[qi], r = qr[qi], v = qv[qi];
        if (v >= 0) {
            for (int i = l; i <= r; i++) {
                ans[i] += v;
            }
#pragma GCC ivdep
            for (int i = l; i <= r; i++) {
                ans[i] = min(ans[i], arr[i]);
            }
        } else {
            for (int i = l; i <= r; i++) {
                ans[i] = max(0, ans[i] + v);
            }
        }
    }
    return vector<int>(ans, ans + n);
}