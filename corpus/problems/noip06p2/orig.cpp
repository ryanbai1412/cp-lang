// corpus/submissions/5165705.cpp with each zero-initialised VLA cleared by memset: clang rejects `T a[n] {}`.
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

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

void solve() {
    int kv, n;
    cin >> kv >> n;
    int arr[n][3];
    for (auto& [w, v, p] : arr) {
        cin >> w >> v >> p;
        p--;
    }
    vector<int> graph[n];
    for (int i = 0; i < n; i++) {
        auto& [w, v, p] = arr[i];
        v *= w;
        if (p != -1) {
            graph[p].push_back(i);
        }
    }
    long dp[kv + 1];
    memset(dp, 0, sizeof dp);
    for (int i = 0; i < n; i++) {
        auto& [w, v, p] = arr[i];
        if (p != -1) {
            continue;
        }
        for (int j = kv; j >= 0; j--) {
            for (int mask = 0; mask < (1 << sz(graph[i])); mask++) {
                int cw = w;
                long cv = v;
                for (int k = 0; k < sz(graph[i]); k++) {
                    if (on(mask, k)) {
                        int u = graph[i][k];
                        cw += arr[u][0];
                        cv += arr[u][1];
                    }
                }
                if (j - cw >= 0) {
                    dp[j] = max(dp[j], dp[j - cw] + cv);
                }
            }
        }
    }
    cout << dp[kv] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}