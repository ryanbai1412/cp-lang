//region template
#include "bits/extc++.h"

using namespace std;

template<class T>
void dbgh(T t) {
	cerr << t << endl;
}

template<class T, class... U>
void dbgh(T t, U... u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; dbgh(__VA_ARGS__); cerr
#else
#define cerr if(false) cerr
#define dbg(...) cerr
#endif

using ld = long double;

#define long int64_t
#define sz(x) static_cast<int>((x).size())
//endregion

const int maxn = 5412, tmaxn = 2e6, tsmaxn = 1 << 23;

int n, k;
int arr[maxn], nxt[maxn];
ld memo1[maxn][maxn], memo2[maxn];

inline ld score(int x) {
	if(k == 4) {
		return ld(x) * x;
	}
	ld c = sqrt(x);
	dbg(c * c * c);
	return c * c * c;
}

ld dp2(int ind);

ld dp1(int ind, int len) {
	if(memo1[ind][len] != -1) {
		return memo1[ind][len];
	}
	ld ans = score(len) + dp2(ind + 1);
	if(nxt[ind] < n) {
		ans = max(ans, dp1(nxt[ind], len + 1));
	}
	return memo1[ind][len] = ans;
}

ld dp2(int ind) {
	if(ind == n) {
		return 0;
	}else if(memo2[ind] != -1) {
		return memo2[ind];
	}
	return memo2[ind] = dp1(ind, 1);
}

void solve() {
	cin >> k >> n;
	if(k == 2) {
		cout << ld(n) << endl;
		return;
	}
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
		arr[i]--;
	}
	ld dp[n + 1];
	dp[n] = 0;
	vector<int> inds[n];
	for(int i = n - 1; i >= 0; i--) {
		dp[i] = score(1) + dp[i + 1];
		int m = sz(inds[arr[i]]);
		for(int j = 1; j <= m; j++) {
			dp[i] = max(dp[i], score(j + 1) + dp[inds[arr[i]][m - j] + 1]);
		}
		inds[arr[i]].push_back(i);
	}
	cout << dp[0] << endl;
}

int main() {
	cout << setprecision(6) << fixed;
	cin.exceptions(ios::failbit);
	cin.tie(0)->sync_with_stdio(0);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	solve();
}