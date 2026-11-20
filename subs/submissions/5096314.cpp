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

clock_t start;
int n, ans;
string t;
set<string> vis;
vector<pair<string, string>> rules;

void answer() {
    if (ans == 1e9) {
        cout << "NO ANSWER!" << endl;
    } else {
        cout << ans << endl;
    }
}

void dfs(const string& s, int d = 0) {
    if (clock() - start > 0.9 * CLOCKS_PER_SEC) {
        answer();
        exit(0);
    }
    if (s == t) {
        ans = min(ans, d);
        return;
    } else if (d == 10 || !vis.insert(s).second) {
        return;
    }
    for (auto& [r1, r2] : rules) {
        for (int i = 0; i < sz(s); i++) {
            if (s.substr(i, sz(r1)) == r1) {
                dfs(s.substr(0, i) + r2 + s.substr(i + sz(r1)), d + 1);
            }
        }
    }
}

void solve() {
    start = clock();
    string s;
    cin >> s >> t;
    string r1, r2;
    while(cin >> r1 >> r2) {
        rules.emplace_back(r1, r2);
    }
    ans = 1e9;
    dfs(s);
    answer();
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    solve();
}