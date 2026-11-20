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

const int maxk = 30;

vector<int> graph[maxk];
int memo[maxk];

int dfs(int u) {
	int &ans = memo[u];
	if (ans != -1) {
		return ans;
	}
	ans = 0;
	for (auto& v : graph[u]) {
		ans = max(ans, dfs(v));
	}
	ans++;
	return ans;
}

void solve() {
	int n, k;
	cin >> n >> k;
	bool arr[k][k];
	memset(arr, 1, sizeof(arr));
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
		for (int a = 0; a < k; a++) {
			for (int b = 0; b < a; b++) {
				arr[s[a] - 'A'][s[b] - 'A'] = false;
			}
		}
	}
	for (int i = 0; i < k; i++) {
		for (int j = 0; j < k; j++) {
			if (arr[i][j]) {
				graph[i].push_back(j);
			}
		}
	}
	memset(memo, -1, sizeof(memo));
	int ans = 0;
	for (int i = 0; i < k; i++) {
		ans = max(ans, dfs(i));
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}