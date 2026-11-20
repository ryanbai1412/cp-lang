#include "bits/extc++.h"

using namespace std;

using ld = long double;

#define sz(x) static_cast<int>((x).size())
#define long int64_t
#define endl "\n"

const int dx[] {-1, 1, 0, 0}, dy[] {0, 0, -1, 1};

int n, m, ba, bb;
map<vector<vector<int>>, ld> memo1, memo2;

inline bool ibs(int x, int y) {
	return 0 <= x && x < n && 0 <= y && y < m;
}

ld dp2(const vector<vector<int>> &arr);

ld dp1(const vector<vector<int>> &arr) {
	if(memo1.count(arr)) {
		return memo1[arr];
	}
	vector<ld> moves;
	for(int i = 0; i < n; i++) {
		for(int j = 0; j < m; j++) {
			if(arr[i][j] == 1) {
				for(int k = 0; k < 4; k++) {
					int cx = i + dx[k];
					int cy = j + dy[k];
					if(ibs(cx, cy) && arr[cx][cy]) {
						auto tmp = arr;
						tmp[cx][cy] = arr[i][j];
						tmp[i][j] = 0;
						moves.push_back(1 - dp2(tmp));
					}
				}
			}
		}
	}
	sort(begin(moves), end(moves), greater<>());
	if(moves.empty()) {
		return memo1[arr] = 0;
	}
	vector<ld> ps(moves.begin(), moves.begin() + min(sz(moves), ba));
	ld sum = 0;
	for(auto &a: ps) {
		sum += a;
	}
	return memo1[arr] = sum / sz(ps);
}

ld dp2(const vector<vector<int>> &arr) {
	if(memo2.count(arr)) {
		return memo2[arr];
	}
	vector<ld> moves;
	for(int i = 0; i < n; i++) {
		for(int j = 0; j < m; j++) {
			if(arr[i][j] == 2) {
				for(int k = 0; k < 4; k++) {
					int cx = i + dx[k];
					int cy = j + dy[k];
					if(ibs(cx, cy) && arr[cx][cy]) {
						auto tmp = arr;
						tmp[cx][cy] = arr[i][j];
						tmp[i][j] = 0;
						moves.push_back(1 - dp1(tmp));
					}
				}
			}
		}
	}
	sort(begin(moves), end(moves), greater<>());
	if(moves.empty()) {
		return memo2[arr] = 0;
	}
	vector<ld> ps(moves.begin(), moves.begin() + min(sz(moves), bb));
	ld sum = 0;
	for(auto &a: ps) {
		sum += a;
	}
	return memo2[arr] = sum / sz(ps);
}

void solve() {
	cin >> n >> m;
	vector<vector<int>> arr(n, vector<int>(m));
	for(int i = 0; i < n; i++) {
		string s;
		cin >> s;
		for(int j = 0; j < m; j++) {
			if(s[j] == 'J') {
				arr[i][j] = 1;
			}else {
				arr[i][j] = 2;
			}
		}
	}
	cin >> ba >> bb;
	cout << dp1(arr) << endl;
}

int main() {
	cout << setprecision(3) << fixed;
	cin.tie(0)->sync_with_stdio(0);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	solve();
}