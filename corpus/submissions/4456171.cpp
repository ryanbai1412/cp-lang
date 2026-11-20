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
	int n, m, q;
	cin >> n >> m >> q;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int req[m] {};
	while (q--) {
		int a, b;
		cin >> a >> b;
		req[a] = max(req[a], b);
	}
	int nxtv[m];
	memset(nxtv, -1, sizeof(nxtv));
	int nxt[n];
	for (int i = n - 1; i >= 0; i--) {
		nxt[i] = nxtv[arr[i]];
		nxtv[arr[i]] = i;
	}
	int ptr[m], cans = 0;
	for (int i = 0; i < m; i++) {
		if (!req[i]) {
			continue;
		}
		int j = nxtv[i];
		for (int k = 1; k < req[i]; k++) {
			j = j == -1 ? -1 : nxt[j];
		}
		if (j == -1) {
			cout << "impossible" << endl;
			return;
		}
		ptr[i] = j;
		cans = max(cans, j);
	}
	int ans = cans + 1;
	for (int i = 0; i < n; i++) {
		if (req[arr[i]]) {
			ptr[arr[i]] = nxt[ptr[arr[i]]];
			if (ptr[arr[i]] == -1) {
				break;
			}
			cans = max(cans, ptr[arr[i]]);
		}
		ans = min(ans, cans - i);
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}