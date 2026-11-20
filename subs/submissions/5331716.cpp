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

void solve() {
    int m, n, kv;
    long vl, ve, vse, mt;
    cin >> m >> n >> kv >> vl >> ve >> vse >> mt;
    m--;
    kv -= n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    int up[n - 1];
    priority_queue<pair<int, int>> pq;
    auto p_fdiv = [&](long a, long b) -> long {
        assert(b > 0);
        if (a < 0) {
            return -1;
        }
        return a / b;
    };
    auto go = [&](int i) -> void {
        long furth = p_fdiv(mt - ve * arr[i] - vse * (up[i] + 1 - arr[i]), vl);
        furth = min(furth, long(arr[i + 1] - up[i] - 2));
        if (furth >= 0) {
            pq.emplace(furth + 1, i);
        }
    };
    for (int i = 0; i < n - 1; i++) {
        long furth = p_fdiv(mt - ve * arr[i], vl);
        if (furth < 0) {
            up[i] = -1;
        } else {
            up[i] = int(min(long(arr[i + 1] - 1), arr[i] + furth));
            go(i);
        }
    }
    while (sz(pq) && kv) {
        auto [f, i] = pq.top();
        pq.pop();
        kv--;
        up[i] += f;
        go(i);
    }
    int ans = 0;
    for (int i = 0; i < n - 1; i++) {
        if (up[i] == -1) {
            continue;
        }
        dbg(arr[i], up[i]);
        ans += up[i] - arr[i] + 1;
    }
    if (ve * m <= mt) {
        ans++;
    }
    cout << ans - 1 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}