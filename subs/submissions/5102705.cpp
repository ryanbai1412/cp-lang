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

struct Bit {
    int n;
    vector<long> v;

    Bit(int n) : n(n), v(n + 1) {}

    void update(int ind, long x) {
        ind++;
        while (ind <= n) {
            v[ind] += x;
            ind += ind & -ind;
        }
    }

    long psum(int ind) const {
        long ans = 0;
        while (ind > 0) {
            ans += v[ind];
            ind -= ind & -ind;
        }
        return ans;
    }

    long query(int l, int r) const {
        return psum(r + 1) - psum(l);
    }
};

struct Frac {
    long num, den;

    friend Frac operator-(const Frac& f) {
        return {-f.num, f.den};
    }

    bool operator<(const Frac& f) const {
        return num * f.den < den * f.num;
    }
};

void solve() {
    int n;
    long kv;
    cin >> n >> kv;
    array<int, 4> arr[n];
    for (auto& a : arr) {
        cin >> a[0] >> a[1];
    }
    for (int i = 0; i < n; i++) {
        arr[i][3] = i;
    }
    sort(arr, arr + n, [&](const auto& a, const auto& b) -> bool {
        return a[1] < b[1];
    });
    for (int i = 0; i < n; i++) {
        arr[i][2] = i;
    }
    sort(arr, arr + n, [&](const auto& a, const auto& b) -> bool {
        return Frac {a[0], a[1]} < Frac {b[0], b[1]};
    });
    tuple<int, Frac, int> ans {-1, {-1, -1}, -1};
    Bit bit1(n), bit2(n);
    for (auto& [s, q, qi, _] : arr) {
        bit1.update(qi, q);
        bit2.update(qi, 1);
        long can = kv * q / s;
        int l = 0, r = n - 1;
        while (l < r) {
            int mid = (l + r + 1) / 2;
            if (bit1.query(0, mid) <= can) {
                l = mid;
            } else {
                r = mid - 1;
            }
        }
        ans = max(ans, tuple<int, Frac, int> {bit2.query(0, l),
                                        -Frac {bit1.query(0, l) * s, q}, qi});
    }
    int vis[n];
    memset(vis, -1, sizeof(vis));
    auto& [cans, _w, qind] = ans;
    for (auto& [_s, _q, qi, i]: arr) {
        vis[qi] = i;
        if (qind == qi) {
            vector<int> vans;
            for (int j = 0; j < n && sz(vans) < cans; j++) {
                if (vis[j] != -1) {
                    vans.push_back(vis[j]);
                }
            }
            cout << sz(vans) << endl;
            for (auto& a : vans) {
                cout << a + 1 << endl;
            }
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}