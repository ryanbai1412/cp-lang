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

constexpr long mod = 10007;

void solve() {
    int n;
    cin >> n;
    vector<int> graph[n];
    for (int i = 0; i < n - 1; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    long arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    long ans_sm = 0, ans_mx = 0;
    for (int i = 0; i < n; i++) {
        vector<long> cv;
        for (auto& a : graph[i]) {
            cv.push_back(arr[a]);
        }
        sort(begin(cv), end(cv), greater<>());
        if (sz(cv) >= 2) {
            ans_mx = max(ans_mx, cv[0] * cv[1]);
        }
        long csm = accumulate(begin(cv), end(cv), long(0));
        ans_sm += csm * csm;
        ans_sm -= arr[i] * arr[i] * sz(graph[i]);
        ans_sm %= mod;
    }
    ans_sm = (ans_sm % mod + mod) % mod;
    cout << ans_mx << " " << ans_sm << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}