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
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

namespace s1 {

bool solve(int ql,
           int qr,
           const vector<pair<int, int>>& arr,
           const vector<int>& queries) {
    if (sz(arr) > 2000 || sz(queries) > 2000) {
        return false;
    }
    for (auto& x : queries) {
        long r = x;
        for (auto& [t, a] : arr) {
            if (t == 1) {
                r += a;
            } else if (t == 2) {
                r -= a;
            } else if (t == 3) {
                r *= a;
            } else {
                r += a * x;
            }
            r = clamp(r, long(ql), long(qr));
        }
        cout << r << endl;
    }
    return true;
}

}  // namespace s1

void solve() {
    int n, ql, qr;
    cin >> n >> ql >> qr;
    vector<pair<int, int>> arr(n);
    for (auto& [t, a] : arr) {
        cin >> t >> a;
    }
    int q;
    cin >> q;
    vector<int> queries(q);
    for (auto& a : queries) {
        cin >> a;
    }
    bool solved = s1::solve(ql, qr, arr, queries);
    assert(solved);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}