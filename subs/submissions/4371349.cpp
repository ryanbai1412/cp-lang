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
	dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 2e5 + 5;

bool arr[maxn];
long cost[maxn], memo[maxn][2][2];
vector<int> graph[maxn];

void pdfs(int u) {
	for (auto& v : graph[u]) {
		graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
		pdfs(v);
	}
}

long dp(int u, bool p, bool used) {
	p |= arr[u];
	if (p && !sz(graph[u])) {
		return 0;
	}
	long& ans = memo[u][p][used];
	if (ans != -1) {
		return ans;
	}
	ans = 1e10;
	long mcost = used ? 0 : cost[u];
	// I do nothing, children bring themselves up
	{
		long tot = 0;
		for (auto& v : graph[u]) {
			tot += dp(v, false, false);
		}
		for (auto& v : graph[u]) {
			long cans = tot - dp(v, false, false);
			if (!p) {
				cans += cost[v] + dp(v, false, true);
			} else {
				cans += dp(v, false, false);
			}
			ans = min(ans, cans);
		}
	}
	// one child bring themselves up and I power the rest
	{
		long tot = 0;
		for (auto& v : graph[u]) {
			tot += dp(v, true, false);
		}
		for (auto& v : graph[u]) {
			ans = min(ans, tot - dp(v, true, false) + cost[v] + dp(v, false, true) + mcost);
		}
	}
	// I power
	if (p) {
		long cans = mcost;
		for (auto& v : graph[u]) {
			cans += dp(v, true, false);
		}
		ans = min(ans, cans);
	}
	return ans;
}

void solve() {
	int n;
	cin >> n;
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	string s;
	cin >> s;
	for (int i = 0; i < n; i++) {
		arr[i] = s[i] == 'Y';
	}
	for (int i = 0; i < n; i++) {
		cin >> cost[i];
	}
	pdfs(0);
	memset(memo, -1, sizeof(memo));
	cout << dp(0, false, false) << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}