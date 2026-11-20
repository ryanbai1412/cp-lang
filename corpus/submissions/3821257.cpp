#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

const int maxn = 1000;

namespace answer {
	int mans = 0, cur = 0;
	vector<tuple<int, int, int>> ans;

	void land(int u) {
		cur++;
		mans = max(mans, cur);
		if(u != -1) {
			ans.emplace_back(0, u, 0);
		}
	}

	void bring(int u) {
		cur--;
		if(u != -1) {
			ans.emplace_back(1, u, 0);
		}
	}

	void move(int u, int v) {
		if(u != -1 && v != -1) {
			ans.emplace_back(2, u, v);
		}
	}

	void print() {
		cout << mans << endl;
		cout << sz(ans) << endl;
		for(auto &[t, a, b]: ans) {
			cout << "LBM"[t] << " " << a + 1;
			if(t == 2) {
				cout << " " << b + 1;
			}
			cout << endl;
		}
	}
}

using answer::land, answer::bring, answer::move;

int memo[maxn];
vector<int> graph[maxn];

int pdfs(int u) {
	int &ans = memo[u];
	if(ans != -1) {
		return ans;
	}

	if(!sz(graph[u])) {
		ans = 1;
		return ans;
	}

	for(auto &v: graph[u]) {
		graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
		pdfs(v);
	}

	sort(begin(graph[u]), end(graph[u]), [&](int a, int b) -> bool {
		return pdfs(a) > pdfs(b);
	});

	ans = pdfs(graph[u][0]);
	if(sz(graph[u]) > 1) {
		ans = max(ans, 1 + pdfs(graph[u][1]));
	}

	return ans;
}

void dfs(int u, int p = -1, bool has = false) {
	if(has) {
		if(!sz(graph[u])) {
			return;
		}else if(sz(graph[u]) == 1) {
			move(u, graph[u][0]);
			dfs(graph[u][0], u, true);
		}else {
			for(int i = 1; i < sz(graph[u]); i++) {
				dfs(graph[u][i], u);
				bring(u);
			}
			move(u, graph[u][0]);
			dfs(graph[u][0], u, true);
		}
		return;
	}
	if(!sz(graph[u])) {
		land(u);
		move(u, p);
		return;
	}

	dfs(graph[u][0], u);

	for(int i = 1; i < sz(graph[u]); i++) {
		dfs(graph[u][i], u);
		bring(u);
	}

	move(u, p);
}

void solve() {
	int n;
	cin >> n;
	int edges[n - 1][2];
	for(int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		edges[i][0] = u;
		edges[i][1] = v;
	}

	auto reset = [&]() -> void {
		memset(memo, -1, sizeof(memo));
		for(int i = 0; i < n; i++) {
			graph[i].clear();
		}
		for(auto &[u, v]: edges) {
			graph[u].push_back(v);
			graph[v].push_back(u);
		}
	};

	pair<int, int> root {1e9, 1e9};
	for(int i = 0; i < n; i++) {
		reset();
		root = min(root, pair<int, int> {pdfs(i), i});
	}
	reset();
	pdfs(root.second);
	dfs(root.second, -1, true);

	answer::print();
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
		//		cout << "Case #" << _ << ": ";
		solve();
	}
}