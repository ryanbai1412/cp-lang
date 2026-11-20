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

void solve() {
	int n, m;
	cin >> n >> m;
	long arr[m][4];
	for (auto& [a, b, c, d] : arr) {
		cin >> a >> b >> c >> d;
		a--;
		b--;
	}
	vector<long> ans(n);
	while (true) {
		vector<long> nxt(n, 1e18);
		for (auto& [u, v, r, p] : arr) {
			nxt[u] = min(nxt[u], max(r, ans[v] - p));
		}
		if (ans == nxt) {
			break;
		}
		swap(ans, nxt);
	}
	for (int i = 0; i < n; i++) {
		if (ans[i] == long(1e18)) {
			ans[i] = -1;
		}
		cout << ans[i] << " \n"[i == n - 1];
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}