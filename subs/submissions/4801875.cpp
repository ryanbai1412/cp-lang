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

const int maxn = 2e3 + 5;

int n, vcnt;
bool graph[maxn][maxn], vis[maxn];
vector<int> st;

void dfs(int u, bool b) {
	if (vis[u]) {
		return;
	}
	vcnt++;
	vis[u] = true;
	st.push_back(u);
	for (int i = 0; i < n; i++) {
		if (graph[u][i] == b && !vis[i]) {
			dfs(i, b);
			st.push_back(u);
		}
	}
}

void solve() {
	cin >> n;
	for (int i = 1; i < n; i++) {
		string s;
		cin >> s;
		for (int j = 0; j < i; j++) {
			graph[i][j] = graph[j][i] = s[j] == 'R';
		}
	}
	auto go = [&](bool b) -> bool {
		vcnt = 0;
		memset(vis, 0, sizeof(vis));
		st.clear();
		dfs(0, b);
		if (vcnt == n) {
			return true;
		}
		return false;
	};
	assert(go(false) || go(true));
	st.pop_back();
	for (int i = 0; i < n; i++) {
		rotate(begin(st), find(begin(st), end(st), i), end(st));
		cout << sz(st) << endl;
		for (int j = 0; j < sz(st); j++) {
			cout << st[j] + 1 << " \n"[j == sz(st) - 1];
		}
	}
}

int main() {
	ios_base::sync_with_stdio(false);
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	solve();
}