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

void solve() {
	int n, k;
	IO::read_int(n, k);
	int m = min(n, k);
	int arr[n];
	for (auto& a : arr) {
		IO::read_int(a);
	}
	if (!k) {
		for (int i = 0; i < n; i++) {
			cout << arr[i] << " \n"[i == n - 1];
		}
		return;
	}
	deque<int> ds;
	{
		int j = 0, csum = 0;
		for (int i = n - 1; i >= 0 && j < m; i--) {
			if (csum += arr[i]) {
				csum--;
			} else {
				ds.push_back(j);
			}
			j++;
		}
	}
	for (int i = 0; i < n; i++) {
		int pi = arr[i];
		while (sz(ds) && ds.back() >= m - i) {
			ds.pop_back();
		}
		arr[i] -= sz(ds);
		arr[i] = max(arr[i], 0);
		if (!binary_search(begin(ds), end(ds), m - i - 1)) {
			arr[i] = max(arr[i], 1);
		}
		ds.push_front(-i - 1);
		for (int j = 0; j < pi; j++) {
			if (sz(ds)) {
				ds.pop_back();
			}
		}
	}
	if (k > n && !*min_element(arr, arr + n)) {
		rotate(arr, arr + n - k % n, arr + n);
	}
	for (int i = 0; i < n; i++) {
		IO::write_int(arr[i], " \n"[i == n - 1]);
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	IO::init();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}