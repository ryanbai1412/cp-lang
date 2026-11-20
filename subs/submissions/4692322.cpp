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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void add_element(string x);
bool check_element(string x);
void compile_set();

int n;
vector<int> ans;

void dfs1(int bit, const vector<int>& cur) {
    if (sz(cur) == 1) {
        return;
    }
    bool vis[n] {};
    for (auto& a : cur) {
        vis[a] = true;
    }
    vector<int> l, r;
    for (auto& a : cur) {
        if ((a >> bit) & 1) {
            string s(n, '0');
            for (int i = 0; i < n; i++) {
                if (!vis[i]) {
                    s[i] = '1';
                }
            }
            s[a] = '1';
            add_element(s);
            l.push_back(a);
        } else {
            r.push_back(a);
        }
    }
    dfs1(bit + 1, l);
    dfs1(bit + 1, r);
}

void dfs2(int bit, const vector<int>& cur) {
    if (sz(cur) == 1) {
        return;
    }
    bool vis[n] {};
    for (auto& a : cur) {
        vis[a] = true;
    }
    vector<int> l, r;
    for (auto& a : cur) {
        string s(n, '0');
        for (int i = 0; i < n; i++) {
            if (!vis[i]) {
                s[i] = '1';
            }
        }
        s[a] = '1';
        if (check_element(s)) {
            ans[a] |= 1 << bit;
            l.push_back(a);
        } else {
            r.push_back(a);
        }
    }
    dfs2(bit + 1, l);
    dfs2(bit + 1, r);
}

vector<int> restore_permutation(int _n, int, int) {
    n = _n;
    ans.resize(n);
    vector<int> arr(n);
    iota(begin(arr), end(arr), 0);
    dfs1(0, arr);
    compile_set();
    dfs2(0, arr);
    return ans;
}