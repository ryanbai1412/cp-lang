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

constexpr int maxn = 45;

int n;
string sa, sb;
vector<int> ans;
vector<pair<string, string>> arr;

void dfs(int ind) {
    if (ind == n) {
        return;
    }
    if (ind && sa == sb) {
        cout << sz(ans) << endl;
        for (auto& a : ans) {
            cout << a + 1 << endl;
        }
        exit(0);
    }
    ans.push_back(0);
    for (int i = 0; i < sz(arr); i++) {
        auto& [a, b] = arr[i];
        sa += a;
        sb += b;
        ans.back() = i;
        dfs(ind + 1);
        sa.resize(sz(sa) - sz(a));
        sb.resize(sz(sb) - sz(b));
    }
    ans.pop_back();
}

void solve() {
    int m;
    cin >> n >> m;
    arr.resize(m);
    for (auto& [a, _] : arr) {
        cin >> a;
    }
    for (auto& [_, b] : arr) {
        cin >> b;
    }
    dfs(0);
    cout << "No solution." << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}