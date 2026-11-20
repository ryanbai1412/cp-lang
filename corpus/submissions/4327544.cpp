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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

namespace IO {
const int BUFFER_SIZE = 1 << 16;

char input_buffer[BUFFER_SIZE];
size_t input_pos = 0, input_len = 0;

char output_buffer[BUFFER_SIZE];
int output_pos = 0;

char number_buffer[100];
uint8_t lookup[100];

void _update_input_buffer() {
	input_len = fread(input_buffer, sizeof(char), BUFFER_SIZE, stdin);
	input_pos = 0;

	if (input_len == 0)
		input_buffer[0] = EOF;
}

inline char next_char(bool advance = true) {
	if (input_pos >= input_len)
		_update_input_buffer();

	return input_buffer[advance ? input_pos++ : input_pos];
}

template <typename T>
inline void read_int(T& number) {
	bool negative = false;
	number = 0;

	while (!isdigit(next_char(false)))
		if (next_char() == '-')
			negative = true;

	do {
		number = 10 * number + (next_char() - '0');
	} while (isdigit(next_char(false)));

	if (negative)
		number = -number;
}

template <typename T, typename... Args>
inline void read_int(T& number, Args&... args) {
	read_int(number);
	read_int(args...);
}

void _flush_output() {
	fwrite(output_buffer, sizeof(char), output_pos, stdout);
	output_pos = 0;
}

inline void write_char(char c) {
	if (output_pos == BUFFER_SIZE)
		_flush_output();

	output_buffer[output_pos++] = c;
}

template <typename T>
inline void write_int(T number, char after = '\0') {
	if (number < 0) {
		write_char('-');
		number = -number;
	}

	int length = 0;

	while (number >= 10) {
		uint8_t lookup_value = lookup[number % 100];
		number /= 100;
		number_buffer[length++] = char((lookup_value & 15) + '0');
		number_buffer[length++] = char((lookup_value >> 4) + '0');
	}

	if (number != 0 || length == 0)
		write_char(char(number + '0'));

	for (int i = length - 1; i >= 0; i--)
		write_char(number_buffer[i]);

	if (after)
		write_char(after);
}

void init() {
	// Make sure _flush_output() is called at the end of the program.
	bool exit_success = atexit(_flush_output) == 0;
	assert(exit_success);

	for (int i = 0; i < 100; i++)
		lookup[i] = uint8_t((i / 10 << 4) + i % 10);
}
}  // namespace IO

const int maxn = 3005;
int tri[2][maxn][maxn], sq[2][maxn][maxn], arr[maxn][maxn];

void solve() {
	int n, k;
	IO::read_int(n, k);
	for (int i = n - 1; i >= 0; i--) {
		for (int j = 0; j < n - i; j++) {
			IO::read_int(arr[i][j]);
		}
	}
	int cind = 0, pind = 1;
	memcpy(sq[cind], arr, sizeof(arr));
	memcpy(tri[cind], arr, sizeof(arr));
	int trix = __lg(k), sqx = __lg((k + 1) / 2);
	for (int i = 1; i <= trix; i++) {
		swap(cind, pind);
		int cx = 1 << (i - 1), cy = 1 << i;
		for (int x = 0; x + cy <= n; x++) {
			for (int y = 0; y + cy <= n; y++) {
				sq[cind][x][y] = max({sq[pind][x][y], sq[pind][x + cx][y], sq[pind][x][y + cx],
									  sq[pind][x + cx][y + cx]});
				tri[cind][x][y] = max({
					tri[pind][x + cx][y],
					tri[pind][x][y + cx],
					sq[pind][x][y],
				});
			}
		}
	}
	int tind = cind;
	memcpy(sq[cind], arr, sizeof(arr));
	for (int i = 1; i <= sqx; i++) {
		swap(cind, pind);
		int cx = 1 << (i - 1), cy = 1 << i;
		for (int x = 0; x + cy <= n; x++) {
			for (int y = 0; y + cy <= n; y++) {
				sq[cind][x][y] = max({sq[pind][x][y], sq[pind][x + cx][y], sq[pind][x][y + cx],
									  sq[pind][x + cx][y + cx]});
			}
		}
	}
	long ans = 0;
	dbg(trix, sqx, tri[tind][0][0]);
	int triadd = k - (1 << trix);
	int sqadd = (k + 1) / 2 - (1 << sqx);
	for (int i = 0; i + k <= n; i++) {
		for (int j = 0; j + k <= n - i; j++) {
			dbg(i, j, tri[tind][i][j + triadd], triadd);
			ans += max({sq[cind][i][j], sq[cind][i + sqadd][j], sq[cind][i][j + sqadd],
						sq[cind][i + sqadd][j + sqadd], tri[tind][i + triadd][j],
						tri[tind][i][j + triadd]});
		}
	}
	IO::write_int(ans, '\n');
}

int main() {
	IO::init();
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}