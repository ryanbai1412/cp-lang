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

struct custom_hash {
	static uint64_t splitmix64(uint64_t x) {
		// http://xorshift.di.unimi.it/splitmix64.c
		x += 0x9e3779b97f4a7c15;
		x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9;
		x = (x ^ (x >> 27)) * 0x94d049bb133111eb;
		return x ^ (x >> 31);
	}
	
	size_t operator()(uint64_t x) const {
		static const uint64_t FIXED_RANDOM = chrono::steady_clock::now().time_since_epoch().count();
		return splitmix64(x + FIXED_RANDOM);
	}
};

const int maxn = 255;

int n, arr[maxn];
__gnu_pbds::gp_hash_table<long, __gnu_pbds::null_type, custom_hash> vis;
vector<int> ans, memo[maxn][maxn];

void dp(int i, int csm, int x) {
	if (i == n) {
		if (csm == 0) {
			ans.push_back(x);
		}
		return;
	}
	if (csm > min(i, n - i)) {
		return;
	}
	long k = x;
	k = k * maxn + i;
	k = k * maxn + csm;
	if (vis.find(k) != vis.end()) {
		return;
	}
	vis.insert(k);
	dp(i + 1, csm, x);
	dp(i + 1, csm + 1, x + arr[i] * arr[i]);
	if (csm) {
		dp(i + 1, csm - 1, x - arr[i] * arr[i]);
	}
}

void solve() {
	cin >> n;
	string s;
	cin >> s;
	for (int i = 0; i < n; i++) {
		arr[i] = s[i] - 'A' + 1;
	}
	dp(0, 0, 0);
	sort(begin(ans), end(ans));
	cout << sz(ans) << endl;
	for (auto& a : ans) {
		cout << a << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}