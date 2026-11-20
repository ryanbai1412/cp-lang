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

const int maxn = 100;

int n, m, arr[maxn], cnt[maxn];
long ans, cur[maxn];

void solve(int ind) {
	int xcnt = 0;
	for (int i = 0; i < m; i++) {
		xcnt += !cnt[i];
	}
	if (xcnt > n - ind) {
		return;
	}
	if (ind == n) {
		ans = max(ans, accumulate(cur, cur + m, long(0)));
		return;
	}
	for (int i = 0; i < m; i++) {
		cnt[i]++;
		long prev = cur[i];
		cur[i] *= arr[ind];
		solve(ind + 1);
		cnt[i]--;
		cur[i] = prev;
	}
}

void solve() {
	assert(n <= 10);
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	fill(begin(cur), end(cur), 1);
	ans = -8e18;
	solve(0);
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	while (true) {
		cin >> n >> m;
		if (n && m) {
			solve();
		} else {
			break;
		}
	}
}