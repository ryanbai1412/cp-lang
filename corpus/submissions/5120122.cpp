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

int n;

bool query(int l, int r) {
    cout << "Look " << l << " " << r << endl;
    char ans[6];
    cin >> ans;
    return ans[0] == 'T';
}

void answer(int v, int p) {
    cout << "Guess " << v << " " << p << endl;
}

void dfs(vector<pair<int, int>> arr) {
    if (sz(arr) == 1) {
        answer(arr[0].second, arr[0].first);
        return;
    }
    sort(begin(arr), end(arr));
    int x = arr[(sz(arr) - 1) / 2].first;
    bool b = query(0, x);
    vector<pair<int, int>> narr;
    for (auto& [p, v] : arr) {
        dbg(p, v);
        if ((b && p <= x) || (!b && p > x)) {
            narr.emplace_back((p + v) % n, v);
        }
    }
    dfs(narr);
}

void solve() {
    cout << "GetSize" << endl;
    cin >> n;
    mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
    int x;
    do {
        x = int(cowng() % n);
    } while (!query(x, x));
    vector<pair<int, int>> arr(n);
    for (int i = 0; i < n; i++) {
        arr[i] = {(x + i) % n, i};
    }
    dfs(arr);
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}