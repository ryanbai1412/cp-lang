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

vector<int> facts, ans;
vector<pair<int, int>> fact;
set<array<int, 3>> vis;

void dfs(int ind, int x) {
    if (ind == sz(fact)) {
        facts.push_back(x);
        return;
    }
    dfs(ind + 1, x);
    auto& [p, e] = fact[ind];
    for (int i = 0; i < e; i++) {
        x *= p;
        dfs(ind + 1, x);
    }
}

void dfs2(int ind, int x, int s) {
    if (x == 1) {
        ans.push_back(s);
        return;
    } else if (x < facts[ind] || ind == sz(facts)) {
        return;
    }
    if (!vis.insert({ind, x, s}).second) {
        return;
    }
    dfs2(ind + 1, x, s);
    while (x % facts[ind] == 0) {
        x /= facts[ind];
        s += facts[ind] - 1;
        dfs2(ind + 1, x, s);
    }
}

void solve() {
    int n;
    cin >> n;
    ([&](int n) -> void {
        for (int i = 2; i < int(1e5); i++) {
            int e = 0;
            while (n % i == 0) {
                n /= i;
                e++;
            }
            if (e) {
                fact.emplace_back(i, e);
            }
        }
    })(n);
    dfs(0, 1);
    sort(begin(facts), end(facts));
    facts.erase(facts.begin());
    dfs2(0, n, 0);
    sort(begin(ans), end(ans));
    ans.erase(unique(begin(ans), end(ans)), ans.end());
    cout << sz(ans) << endl;
    for (int i = 0; i < sz(ans); i++) {
        cout << ans[i] << " \n"[i == sz(ans) - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}