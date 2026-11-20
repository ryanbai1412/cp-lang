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

const int maxn = 3e3 + 5;

vector<int> graph[maxn];
bool good;
int n, k, q, arr[maxn], vis[maxn];

void dfs(int u) {
	if (vis[u]) {
		if (vis[u] == 1) {
			good = false;
		}
		return;
	}
	vis[u] = 1;
	for (int v : graph[u]) {
		dfs(v);
	}
	vis[u] = 2;
}

bool solve(int l, int r) {
	for (int i = 0; i < k; i++) {
		graph[i].clear();
		vis[i] = 0;
	}
	for (int i = l; i < r; i++) {
		if ((i ^ l) & 1) {
			graph[arr[i]].push_back(arr[i + 1]);
		} else {
			graph[arr[i + 1]].push_back(arr[i]);
		}
	}
	good = true;
	for (int i = 0; i < k; i++) {
		dfs(i);
	}
	return good;
}

void solve() {
	cin >> n >> k >> q;
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
		arr[i]--;
	}
	int ans[n];
	for (int i = 0; i < n; i++) {
		int l = i, r = n;
		while (l < r) {
			int mid = (l + r) / 2;
			if (!solve(i, mid)) {
				r = mid;
			} else {
				l = mid + 1;
			}
		}
		ans[i] = l;
	}
	while (q--) {
		int l, r;
		cin >> l >> r;
		l--;
		r--;
		if (r >= ans[l]) {
			cout << "NO" << endl;
		} else {
			cout << "YES" << endl;
		}
	}
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}