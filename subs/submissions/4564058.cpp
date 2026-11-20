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

namespace s1 {
const int maxn = 10;

int n, arr[maxn], poss[maxn];
bool vis[maxn];
vector<int> vals;

void dfs(int ind) {
	if (ind == n) {
		for (int i = 0; i < n; i++) {
			int prv = i ? i - 1 : n - 1;
			int nxt = i + 1 < n ? i + 1 : 0;
			if (min(vals[prv], vals[nxt]) != arr[i]) {
				return;
			}
		}
		for (int i = 0; i < n; i++) {
			poss[i] |= 1 << vals[i];
		}
		return;
	}
	int prv = arr[ind ? ind - 1 : n - 1];
	int nxt = arr[ind + 1 < n ? ind + 1 : 0];
	for (int i = 0; i < n; i++) {
		if (!vis[i] && i >= max(prv, nxt)) {
			vis[i] = true;
			vals.push_back(i);
			dfs(ind + 1);
			vis[i] = false;
			vals.pop_back();
		}
	}
}

void solve(const vector<int> &arra) {
	copy(begin(arra), end(arra), arr);
	n = sz(arra);
	memset(poss, 0, sizeof(poss));
	dfs(0);
	for (int i = 0; i < n; i++) {
		if (__builtin_popcount(poss[i]) == 1) {
			cout << __builtin_ctz(poss[i]) + 1;
		} else {
			cout << 0;
		}
		cout << " \n"[i == n - 1];
	}
}
}

void solve() {
	int n;
	cin >> n;
	vector<int> arr(n);
	for (auto& a : arr) {
		cin >> a;
		a--;
	}
	if (sz(arr) <= 9) {
		s1::solve(arr);
		return;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int tcs;
	cin >> tcs;
	while (tcs--) {
		solve();
	}
}