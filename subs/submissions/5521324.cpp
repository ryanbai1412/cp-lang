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
            assert(ql <= r && r <= qr);
            if (t == 1) {
                r += a;
            } else if (t == 2) {
                r -= a;
            } else if (t == 3) {
                r *= a;
            } else {
                r += long(a) * x;
            }
            r = clamp(r, long(ql), long(qr));
        }
        cout << r << endl;
    }
    return true;
}

}  // namespace s1

namespace s2 {

constexpr int maxn = 3e5 + 5;

int ql, qr;
vector<pair<int, long>> arr;
map<int, int> memo[maxn];

int clamp(long r) {
    return int(std::clamp(r, long(ql), long(qr)));
}

int dp(int ind, int x) {
    if (ind == sz(arr)) {
        return x;
    }
    auto [it, inserted] = memo[ind].emplace(x, 0);
    if (!inserted) {
        return it->second;
    }
    long r = x;
    auto& [t, a] = arr[ind];
    if (t == 1) {
        r += a;
    } else if (t == 2) {
        r -= a;
    } else if (t == 3) {
        r *= a;
    } else {
        r += long(a) * x;
    }
    return it->second = dp(ind + 1, clamp(r));
}

bool solve(int _ql,
           int _qr,
           const vector<pair<int, int>>& _arr,
           const vector<int>& queries) {
    for (auto& [t, _a] : _arr) {
        if (t == 4) {
            return false;
        }
    }
    ql = _ql;
    qr = _qr;
    for (auto [t, a] : _arr) {
        if (t == 2) {
            t = 1;
            a = -a;
        }
        if (sz(arr) && arr.back().first == 1 && t == 1) {
            arr.back().second += a;
        } else {
            arr.emplace_back(t, a);
        }
    }
    for (auto& x : queries) {
        cout << dp(0, x) << endl;
    }
    return true;
}

}  // namespace s2

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
    bool solved =
        s2::solve(ql, qr, arr, queries) || s1::solve(ql, qr, arr, queries);
    assert(solved);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}