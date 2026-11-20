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
#define sz(x) int((x).size())

struct DS1 {
    long sum = 0;
    multiset<int> s;

    void insert(int x) {
        sum += x;
        s.insert(x);
    }

    void erase(int x) {
        sum -= x;
        s.erase(s.find(x));
    }

    void erase_first() {
        erase(*s.begin());
    }

    void erase_last() {
        erase(*(--s.end()));
    }
};

constexpr int maxn = 4e5 + 5;

vector<int> graph[maxn], arr[maxn];
int iarr[maxn];

void dfs(int u, int d) {
    arr[d].push_back(iarr[u]);
    for (auto& v : graph[u]) {
        dfs(v, d + 1);
    }
}

void solve() {
	int n, x, y;
	cin >> n >> x >> y;
	string s;
	cin >> s;
    for (int i = 0; i < n; i++) {
        cin >> iarr[i];
    }
    vector<int> roots;
    {
        int ind = 0;
        vector<int> st;
        for (auto& a : s) {
            if (a == '(') {
                if (!sz(st)) {
                    roots.push_back(ind);
                } else {
                    graph[st.back()].push_back(ind);
                }
                st.push_back(ind++);
            } else {
                st.pop_back();
            }
        }
    }
    for (auto& a : roots) {
        dfs(a, 0);
    }
    assert(!x && y);
    DS1 ds;
    long ans = 0;
    for (int i = 0; i < n; i++) {
        for (auto& a : arr[i]) {
            ds.insert(a);
        }
        ds.erase_last();
        ans += ds.sum;
    }
    cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}