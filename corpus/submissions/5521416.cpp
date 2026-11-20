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
    int n;
    cin >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    bool vis[n] {};
    vector<vector<int>> cycles;
    for (int i = 0; i < n; i++) {
        if (vis[i]) {
            continue;
        }
        int u = i;
        vector<int> cycle;
        while (!vis[u]) {
            cycle.push_back(u);
            vis[u] = true;
            u = arr[u];
        }
        cycles.push_back(cycle);
    }
    int base[n] {};
    vector<vector<int>> bcycles;
    for (auto& a : cycles) {
        assert(sz(a) % 2 == 0);
        if (sz(a) == 2) {
            base[min(a[0], a[1])]++;
            base[max(a[0], a[1])]--;
        } else {
            bcycles.push_back(a);
        }
    }
    int m = sz(bcycles);
    for (int i = 0; i < (1 << m); i++) {
        int carr[n];
        memcpy(carr, base, sizeof(carr));
        for (int j = 0; j < m; j++) {
            auto& a = bcycles[j];
            for (int k = 0; k < sz(a); k++) {
                carr[k] += ((k ^ on(i, j)) % 2) * 2 - 1;
            }
        }
        int cpsum = 0;
        for (auto& a : carr) {
            cpsum += a;
            if (cpsum < 0) {
                goto loop;
            }
        }
        for (auto& a : carr) {
            if (a == -1) {
                cout << ")";
            } else {
                cout << "(";
            }
        }
        cout << endl;
        return;
    loop:;
    }
    assert(false);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}