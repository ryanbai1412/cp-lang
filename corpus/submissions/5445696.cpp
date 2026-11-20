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

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

long sq(long x) {
    return x * x;
}

struct Circle {
    int x, y, r;
};

struct Cmp {
    int* x;

    pair<ld, ld> eval(const Circle& c) const {
        ld cr = sqrtl(sq(c.r) - sq(*x - c.x));
        return {c.y - cr, c.y + cr};
    }

    bool operator()(const Circle& a, const Circle& b) const {
        return eval(a) < eval(b);
    }

    bool operator()(const Circle& a, int b) const {
        return eval(a).first < b;
    }
};

void solve() {
    int n;
    cin >> n;
    Circle arr[n];
    for (auto& [x, y, r] : arr) {
        cin >> x >> y >> r;
    }
    sort(arr, arr + n, [&](const auto& a, const auto& b) -> bool {
        return a.r > b.r;
    });
    vector<pair<int, int>> events;
    for (int i = 0; i < n; i++) {
        auto& [x, y, r] = arr[i];
        events.emplace_back(x - r, i);
        events.emplace_back(x + r, i);
    }
    int par[n];
    memset(par, -1, sizeof(par));
    sort(begin(events), end(events));
    int cx = -1;
    Cmp cmp {&cx};
    map<Circle, int, Cmp> ts(cmp);
    for (auto& [nx, i] : events) {
        cx = nx;
        auto& [x, y, r] = arr[i];
        if (nx == x + r) {
            ts.erase(arr[i]);
            continue;
        }
        auto it = ts.upper_bound({nx, y, 0});
        if (it != ts.begin()) {
            --it;
            if (y < cmp.eval(it->first).second) {
                par[i] = it->second;
            }
        }
        ts[arr[i]] = i;
    }
    int cnt[n] {};
    for (int i = 0; i < n; i++) {
        if (par[i] != -1) {
            cnt[i] = cnt[par[i]] + 1;
        }
    }
    long ans = 0;
    for (int i = 0; i < n; i++) {
        long cx = sq(arr[i].r);
        if (cnt[i] % 2 == 0) {
            ans += cx;
        } else {
            ans -= cx;
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}