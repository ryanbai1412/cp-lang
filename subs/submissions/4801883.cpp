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

int n, m, k;
vector<pair<int, int>> arra, arrb;

struct chash {
	uint64_t operator()(uint64_t x) const {
		x ^= x << 19;
		x ^= x >> 21;
		x ^= x << 15;
		x ^= x >> 23;
		return x;
	}
};

namespace s1 {

const int maxa = 2005;

int lecs[2][maxa], suff[2][maxa];
__gnu_pbds::gp_hash_table<long, int, chash> memo[2][maxa];

int dp(int r, int t, array<int, 2> vis) {
	if (t >= maxa) {
		return 0;
	}
	for (int i = 0; i < 2; i++) {
		vis[i] = max(vis[i], suff[i][t]);
	}
	long key = 0;
	key = key * maxa + vis[0];
	key = key * maxa + vis[1];
	auto it = memo[r][t].find(key);
	if (it != memo[r][t].end()) {
		return it->second;
	}
	int add = 0;
	if (lecs[r][t] != -1 && vis[r] < lecs[r][t]) {
		vis[r] = lecs[r][t];
		add++;
	}
	int ans = max(dp(r, t + 1, vis), dp(r ^ 1, t + k, vis));
	return memo[r][t][key] = ans + add;
}

void solve() {
	memset(lecs, -1, sizeof(lecs));
	for (auto& [l, r] : arra) {
		assert(0 <= l && l < r && r <= maxa);
		for (int j = l; j < r; j++) {
			assert(lecs[0][j] == -1);
			lecs[0][j] = l;
		}
	}
	for (auto& [l, r] : arrb) {
		assert(0 <= l && l < r && r <= maxa);
		for (int j = l; j < r; j++) {
			assert(lecs[1][j] == -1);
			lecs[1][j] = l;
		}
	}
	for (int i = 0; i < 2; i++) {
		for (int j = 0; j < maxa - 1; j++) {
			suff[i][j + 1] = max(suff[i][j], lecs[i][j]);
		}
		for (int j = 0; j < maxa; j++) {
			if (lecs[i][j] != -1) {
				suff[i][j] = min(suff[i][j], lecs[i][j] - 1);
			}
		}
	}
	cout << dp(0, 0, {-1, -1}) << endl;
}

}  // namespace s1

struct HT {
	const static int n = 1 << 24;
	int k[n] {}, v[n] {};

	int& find(int i) {
		int x = i;
		i ^= i << 11;
		i ^= i >> 17;
		i &= (n - 1);
		for (; k[i] != x && k[i]; i = (i + 1) & (n - 1))
			;
		if (!k[i]) {
			k[i] = x;
		}
		return v[i];
	}
};

HT memo[2];
vector<pair<int, int>> arr[2];

using It = vector<pair<int, int>>::iterator;

auto ub(int r, int x) {
	return upper_bound(begin(arr[r]), end(arr[r]), pair<int, int> {x, 1e9});
}

int dp(int r, int t, It me) {
	dbg(r, t);
	int& ans = memo[r].find(t);
	if (ans) {
		return ans;
	}
	ans = 0;
	assert(me != arr[r].end());
	{
		auto it = next(me);
		if (it != arr[r].end()) {
			ans = dp(r, it->second, it);
		}
	}
	{
		auto it = ub(r ^ 1, t + k);
		if (it != arr[r ^ 1].end()) {
			ans = max(ans, dp(r ^ 1, max(it->second, t + k), it));
		}
	}
	ans++;
	return ans;
}

void solve() {
	cin >> n >> m >> k;
	arra.resize(n);
	arrb.resize(m);
	for (auto& [l, r] : arra) {
		cin >> l >> r;
	}
	for (auto& [l, r] : arrb) {
		cin >> l >> r;
	}
	int mx = 0;
	for (auto& [l, r] : arra) {
		mx = max(mx, r);
	}
	for (auto& [l, r] : arrb) {
		mx = max(mx, r);
	}
	if (mx <= 2005) {
		s1::solve();
		return;
	}
	arr[0] = arra;
	arr[1] = arrb;
	for (auto& a : arr) {
		for (auto& [l, r] : a) {
			swap(l, r);
		}
		sort(begin(a), end(a));
	}
	int ans = dp(0, arr[0][0].second, arr[0].begin());
	auto it = ub(1, k);
	if (it != arr[1].end()) {
		ans = max(ans, dp(1, max(it->second, k), it));
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}