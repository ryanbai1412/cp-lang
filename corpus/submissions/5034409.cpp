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

struct Node {
    bool leaf;
    int mx, ch[26];
} heap[1 << 19];

int hind = 1;
vector<char> ans;

void insert(int& o, string_view s) {
    if (!o) {
        o = hind++;
    }
    heap[o].mx = max(heap[o].mx, sz(s) + 1);
    if (!sz(s)) {
        heap[o].leaf = true;
        return;
    }
    insert(heap[o].ch[s[0] - 'a'], s.substr(1));
}

void dfs(int o, bool mx) {
    if (heap[o].leaf) {
        ans.push_back('P');
    }
    pair<int, int> opt {};
    for (int i = 0; i < 26; i++) {
        opt = max(opt, pair<int, int> {heap[heap[o].ch[i]].mx, i});
    }
    int u = opt.second;
    if (!mx) {
        u = -1;
    }
    for (int i = 0; i < 26; i++) {
        if (heap[o].ch[i] && i != u) {
            ans.push_back(char('a' + i));
            dfs(heap[o].ch[i], false);
            ans.push_back('-');
        }
    }
    if (u != -1 && heap[o].ch[u]) {
        ans.push_back(char('a' + u));
        dfs(heap[o].ch[u], true);
    }
}

void solve() {
    int n;
    cin >> n;
    int root = 0;
    for (int i = 0; i < n; i++) {
        string s;
        cin >> s;
        insert(root, s);
    }
    dfs(root, true);
    cout << sz(ans) << endl;
    for (auto& a : ans) {
        cout << a << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}