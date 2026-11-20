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

const int maxn = 2e5 + 5;

int d;
vector<int> graph[maxn];

pair<int, int> dfs(int u) {
	if (!sz(graph[u])) {
		return {1, 0};
	}
	vector<pair<int, int>> ch;
	for (auto& v : graph[u]) {
		auto x = dfs(v);
		x.second++;
		ch.push_back(x);
	}
	sort(begin(ch), end(ch), [&](const auto &a, const auto &b) -> bool {
		return a.second > b.second;
	});
	int ans = 0, mn = 1e9;
	for (auto& [x, dist] : ch) {
		ans += x - 1;
		if (dist + mn >= d) {
			ans++;
			mn = dist;
		}
	}
	if (mn >= d) {
		ans++;
		mn = 0;
	}
	return {ans, mn};
}

void solve() {
	int n;
	cin >> n >> d;
	for (int i = 1; i < n; i++) {
		int x;
		cin >> x;
		graph[x].push_back(i);
	}
	cout << dfs(0).first << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}