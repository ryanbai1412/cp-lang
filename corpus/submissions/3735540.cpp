#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long __int128

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

vector<int> sent;

#ifdef LOCAL

void send(int x) {
	sent.push_back(x);
}

void output(int x) {
	cout << x << endl;
}

#endif

const int maxn = 40;

long dp[maxn + 1][32];

void pcomp() {
	fill(dp[0], dp[0] + 32, 1);
	for(int i = 1; i <= maxn; i++) {
		for(int j = 0; j < 32; j++) {
			for(int k = j; k < 32; k++) {
				dp[i][j] += dp[i - 1][k];
			}
		}
	}
}

vector<int> ans;

void gen(int n, long x) {
	if(n) {
		long prev = sz(ans) ? ans.back() : 0;
		for(int j = prev; j < 32; j++) {
			if(dp[n - 1][j] > x) {
				ans.push_back(j);
				gen(n - 1, x);
				return;
			}
			x -= dp[n - 1][j];
		}
	}
}

vector<int> gen(long x) {
	ans.clear();
	gen(maxn, x);
	return ans;
}

long from(const vector<int> &arr) {
	assert(sz(arr) == maxn);
	long ans = 0;
	for(int i = 0; i < maxn; i++) {
		for(int j = i ? arr[i - 1] : 0; j < arr[i]; j++) {
			ans += dp[maxn - i - 1][j];
		}
	}
	return ans;
}

void encode(int n, int arr[]) {
	if(!dp[0][0]) {
		pcomp();
	}
	for(int i = 0; i < n; i += 8) {
		long chash = 0;
		for(int j = i; j < min(n, i + 8); j++) {
			if(j < n) {
				chash |= long(arr[j]) << ((j - i) << 3);
			}
		}
		dbg(uint64_t(chash));
		for(auto &a: gen(chash)) {
			send(((i >> 3) << 5) | a);
		}
	}
}

void decode(int n, int m, int arr[]) {
	if(!dp[0][0]) {
		pcomp();
	}
	vector<int> message[n];
	for(int i = 0; i < m; i++) {
		message[arr[i] >> 5].push_back(arr[i] & 31);
	}
	for(int i = 0; i < n; i += 8) {
		auto &cm = message[i / 8];
		sort(begin(cm), end(cm));
		long cur = from(cm);
		dbg(uint64_t(cur));
		for(int j = i; j < min(n, i + 8); j++) {
			output(cur & 255);
			cur >>= 8;
		}
	}
}

#ifdef LOCAL

void solve() {
	pcomp();
	gen(2);
	for(int i = 0; i <= 255; i++) {
		cerr << i << ": ";
		for(auto &a: gen(i)) {
			cerr << a << " ";
		}
		cerr << endl;
	}
	int n;
	cin >> n;
	int arr[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	encode(n, arr);
	int m = sz(sent);
	cout << "USED " << m << endl;
	int encoded[m];
	for(int i = 0; i < m; i++) {
		encoded[i] = sent[i];
	}
	shuffle(encoded, encoded + m, mt19937(chrono::steady_clock::now().time_since_epoch().count()));
	decode(n, m, encoded);
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}

#endif