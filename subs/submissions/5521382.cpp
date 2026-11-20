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

template <typename T>
struct ST {
    int n;
    vector<T> v, arr;

    ST() {}
    ST(const vector<T>& arr) : n(sz(arr)), v(4 * n), arr(arr) {
        build(1, 0, n - 1);
    }

    void build(int o, int l, int r) {
        if (l == r) {
            v[o] = arr[l];
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = v[lc] + v[rc];
    }

    void update(int o, int l, int r, int ind, const T& x) {
        if (l == r) {
            v[o] = x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update(lc, l, mid, ind, x);
        } else {
            update(rc, mid + 1, r, ind, x);
        }
    }

    void update(int ind, const T& x) {
        update(1, 0, n - 1, ind, x);
    }

    T query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                return query(lc, l, mid, ql, qr) +
                       query(rc, mid + 1, r, ql, qr);
            }
            return query(lc, l, mid, ql, qr);
        }
        return query(rc, mid + 1, r, ql, qr);
    }

    T query(int l, int r) const {
        return query(1, 0, n - 1, l, r);
    }
};

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

struct Node {
    long v, mn, mx;

    Node operator+(const Node& n) const {
        return {v + n.v, min(mn, v + n.mn), max(mx, v + n.mx)};
    }
};

constexpr int maxn = 3e5 + 5;

int ql, qr;
vector<pair<int, int>> arr;
ST<Node> st;
map<pair<int, int>, int> memo;

int clamp(long r) {
    return int(std::clamp(r, long(ql), long(qr)));
}

int dp(int ind, int x, bool b) {
    if (ind == sz(arr)) {
        return x;
    }
    auto [it, inserted] = memo.insert({{ind, x}, 0});
    if (!inserted) {
        return it->second;
    }
    auto eval = [&]() -> int {
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
        return it->second = dp(ind + 1, clamp(r), b);
    };
    if (b || arr[ind].first == 3) {
        return eval();
    }
    int l = ind - 1, r = sz(arr) - 1;
    while (l < r) {
        int mid = (l + r + 1) / 2;
        auto [_v, mn, mx] = st.query(ind, mid);
        if (ql <= x + mn && x + mx <= qr) {
            l = mid;
        } else {
            r = mid - 1;
        }
    }
    if (l < ind) {
        return eval();
    }
    return it->second = dp(l + 1, x + st.query(ind, l).v, true);
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
    arr = _arr;
    arr.erase(remove_if(begin(arr), end(arr),
                        [&](const auto& a) -> bool {
                            return a.first == 3 && a.second == 1;
                        }),
              arr.end());
    vector<Node> stv;
    for (auto& [t, a] : arr) {
        if (t == 1) {
            stv.push_back(Node {a, a, a});
        } else if (t == 2) {
            stv.push_back(Node {-a, -a, -a});
        } else if (t == 3) {
            stv.push_back(Node {0, int(-1e9), int(1e9)});
        } else {
            assert(false);
        }
    }
    st = stv;
    for (auto& x : queries) {
        cout << dp(0, x, false) << endl;
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