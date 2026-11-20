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
using min_pq = priority_queue<T, vector<T>, greater<T>>;

void solve() {
    int n, m, k;
    cin >> n >> m >> k;
    vector<pair<int, int>> arr[2];
    arr[0].resize(n);
    arr[1].resize(m);
    for (auto& a : arr) {
        for (auto& [l, r] : a) {
            cin >> l >> r;
        }
        sort(begin(a), end(a));
    }
    int ans = 0, dist[n + m + 1][2][2];
    memset(dist, 0x3f, sizeof(dist));
    min_pq<tuple<int, int, bool, bool>> pq;
    auto push = [&](int u, bool loc, bool other, int d) -> void {
        ans = max(ans, u);
        if (d < dist[u][loc][other]) {
            dist[u][loc][other] = d;
            pq.emplace(d, u, loc, other);
        }
    };
    push(1, false, false, arr[0][0].first);
    for (auto& [l, r] : arr[1]) {
        if (k < r) {
            push(1, true, false, max(l, k));
        }
    }
    while (sz(pq)) {
        auto [d, u, loc, other] = pq.top();
        pq.pop();
        if (dist[u][loc][other] != d) {
            continue;
        }
        auto mit = --lower_bound(begin(arr[loc]), end(arr[loc]),
                                 pair<int, int> {d + 1, -1});
        auto oit = --lower_bound(begin(arr[!loc]), end(arr[!loc]),
                                 pair<int, int> {d + 1, -1});
        dbg(loc, d, mit - arr[loc].begin(), mit->second);
        {
            auto it = next(mit);
            if (it != arr[loc].end()) {
                int t = it->first;
                push(u + 1, loc, other && t < oit->second, t);
            }
        }
        {
            auto it = --lower_bound(begin(arr[!loc]), end(arr[!loc]),
                                    pair<int, int> {d + k + 1, -1});
            if (other && oit == it) {
                ++it;
            }
            if (it != arr[!loc].end()) {
                if (d + k >= it->second) {
                    ++it;
                }
                if (it != arr[!loc].end()) {
                    int t = max(d + k, it->first);
                    push(u + 1, !loc, t < mit->second, t);
                }
            }
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}