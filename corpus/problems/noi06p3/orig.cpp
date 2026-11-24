// Reference for NOI 2006 P3 at the statement's bounds: the AC submission
// corpus/submissions/3868871.cpp with its memoised recursion run bottom-up. Every change:
// - maxn 1e5 -> gone: the arrays sized for the judge's data were indexed past by n <= 1e6.
// - memo[maxn][25][2] (400 MB at maxn = 1e6, against 128 MB) and heights[maxn] become two rows:
//   dp(ind, ...) reads only dp(ind + 1, ...), so the rows ind and ind + 1 are kept, from
//   ind = n - 1 down to 0. This also drops the recursion 1e6 calls deep. At n = 1e6 the
//   widened original took 2 s and 650 MB against the 1 s, 128 MB limit.
// - A height missing from row ind + 1 still costs 4e18, through the same additions.
// Unchanged: the candidate heights (within 2 of a value in rows i - 2..=i + 2), the
// transitions (keep the height, or rise from lo, fall from hi), the base case (the last row
// must be lo), the answer's 1e9 start and min, the mirroring of the left side, and the input
// and output formats.
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
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

const int maxm = 25;
const bool lo = false, hi = true;
const long inf = 4e18;

int n;
vector<int> arr;

void cands(int i, vector<int> &h) {
	h.clear();
	for(int j = max(0, i - 2); j <= min(n - 1, i + 2); j++) {
		for(int k = max(arr[i], arr[j] - 2); k <= arr[j] + 2; k++) {
			h.push_back(k);
		}
	}
	sort(begin(h), end(h));
	h.erase(unique(begin(h), end(h)), h.end());
}

long solve(const vector<int> &_arr) {
	arr = _arr;
	n = sz(arr);

	// nxt[k][cur] = dp(ind + 1, nh[k], cur); dp of a height not in the row is inf.
	vector<int> h, nh;
	long cur_dp[maxm][2], nxt[maxm][2];
	cands(n - 1, nh);
	for(int k = 0; k < sz(nh); k++) {
		nxt[k][lo] = 0;
		nxt[k][hi] = inf;
	}
	for(int ind = n - 2; ind >= 0; ind--) {
		cands(ind, h);
		for(int k = 0; k < sz(h); k++) {
			int height = h[k];
			auto it = lower_bound(begin(nh), end(nh), height);
			for(int c = 0; c < 2; c++) {
				bool cur = c;
				long ans = inf;

				// continue
				ans = min(ans, height - arr[ind + 1] + (it != nh.end() && *it == height ? nxt[it - nh.begin()][cur] : inf));

				// new
				bool next = cur ^ true;
				for(int j = 0; j < sz(nh); j++) {
					int a = nh[j];
					if(next == lo ? a < height : a > height) {
						ans = min(ans, a - arr[ind + 1] + nxt[j][next]);
					}
				}
				cur_dp[k][cur] = ans;
			}
		}
		swap(h, nh);
		memcpy(nxt, cur_dp, sizeof(nxt));
	}

	long ans = 1e9;
	for(int k = 0; k < sz(nh); k++) {
		ans = min(ans, nh[k] - arr[0] + nxt[k][lo]);
	}
	return ans;
}

void solve() {
	cin >> n;
	vector<int> a(n), b(n);
	for(int i = 0; i < n; i++) {
		cin >> a[i] >> b[i];
	}
	int mx = *max_element(begin(a), end(a));
	for(auto &x: a) {
		x = mx - x;
	}
	cout << solve(a) + solve(b) << endl;
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