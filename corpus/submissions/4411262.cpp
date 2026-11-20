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

#define long int64_t
#define sz(x) int((x).size())

bool taller(int a, int b) {
	cout << "c " << a + 1 << " " << b + 1 << endl;
	int x;
	cin >> x;
	assert(x != -1);
	return a == x - 1;
}

int query(int a) {
	cout << "h " << a + 1 << endl;
	int x;
	cin >> x;
	assert(x != -1);
	return x - 1;
}

const int maxn = 1e4 + 5, branch = 1;

bool vis[maxn];

vector<int> qsort(int l, int r, const vector<int> &arr) {
	static mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
	if (sz(arr) <= 5) {
		if (sz(arr) <= 1) {
			return arr;
		} else if (sz(arr) == 2) {
			if (taller(arr[0], arr[1])) {
				return {arr[1], arr[0]};
			} else {
				return arr;
			}
		}
		vector<int> ans = arr;
		sort(begin(ans), end(ans), [&](int a, int b) -> bool {
			return taller(b, a);
		});
		return ans;
	}
	vector<pair<int, int>> partitions;
	for (int i = 0; i < branch; i++) {
		int x;
		do {
			x = arr[cowng() % sz(arr)];
		} while (vis[x]);
		vis[x] = true;
		partitions.emplace_back(0, x);
	}
	for (auto& [a, x] : partitions) {
		a = query(x);
	}
	sort(begin(partitions), end(partitions));
	vector<int> splits[branch + 1];
	for (auto& a : arr) {
		if (!vis[a]) {
			int bl = 0, br = branch;
			while (bl < br) {
				int mid = (bl + br) / 2;
				if (taller(partitions[mid].second, a)) {
					br = mid;
				} else {
					bl = mid + 1;
				}
			}
			splits[bl].push_back(a);
		}
	}
	int prev = l;
	vector<int> ans;
	for (int i = 0; i < branch; i++) {
		vector<int> cur = qsort(prev, partitions[i].first, splits[i]);
		ans.insert(ans.end(), begin(cur), end(cur));
		ans.push_back(partitions[i].second);
		prev = partitions[i].first;
	}
	vector<int> cur = qsort(partitions.back().first, r, splits[branch]);
	ans.insert(ans.end(), begin(cur), end(cur));
	return ans;
}

void solve() {
	int n, q1, q2, t;
	cin >> n >> q1 >> q2 >> t;
	vector<int> perm(n);
	iota(begin(perm), end(perm), 0);
	perm = qsort(0, n - 1, perm);
	vector<int> ans(n);
	for (int i = 0; i < n; i++) {
		ans[perm[i]] = i + 1;
	}
	cout << "!";
	for (auto& a : ans) {
		cout << " " << a;
	}
	cout << endl;
}

int main() {
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}