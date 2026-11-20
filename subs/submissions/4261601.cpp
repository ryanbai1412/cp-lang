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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 1e5 + 5;

vector<vector<int>> parse(const string& s) {
	vector<vector<int>> graph;
	int ind = 0;
	vector<int> stk;
	graph.emplace_back();
	stk.push_back(ind++);
	for (auto& a : s) {
		if (a == '(') {
			int u = ind++;
			graph.emplace_back();
			graph[stk.back()].push_back(u);
			stk.push_back(u);
		} else {
			stk.pop_back();
		}
	}
	return graph;
}

vector<vector<int>> graph;
vector<pair<int, int>> ans;
int siz[maxn];

void pdfs(int u) {
	siz[u] = 2;
	for (auto& v : graph[u]) {
		pdfs(v);
		siz[u] += siz[v];
	}
}

void dfs1(int u, int pre) {
	int csz = pre + siz[u] - 1;
	for (int i = sz(graph[u]) - 1; i >= 0; i--) {
		int v = graph[u][i];
		dfs1(v, csz -= siz[v]);
	}
	ans.emplace_back(6, pre);
}

void dfs2(int u, int pre) {
	ans.emplace_back(5, pre);
	int csz = pre + 1;
	for (auto& v : graph[u]) {
		dfs2(v, csz);
		csz += siz[v];
	}
}

void solve() {
	ans.clear();
	int n, _k;
	cin >> n >> _k;
	string s, t;
	cin >> s >> t;
	{
		graph = parse(s);
		pdfs(0);
		siz[0]--;
		dfs1(0, 0);
		ans.pop_back();
	}
	{
		graph = parse(t);
		pdfs(0);
		int tmp = sz(ans);
		dfs2(0, -1);
		ans.erase(ans.begin() + tmp);
	}
	cout << sz(ans) << endl;
	for (auto& [a, b] : ans) {
		cout << a << " " << b << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int file, t = 1;
	cin >> file >> t;
	assert(!file || (7 <= file && file <= 8) || (14 <= file && file <= 16));
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}