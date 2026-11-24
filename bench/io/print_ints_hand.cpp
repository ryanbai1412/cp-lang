// print_ints.cp by hand: a 64 KiB char buffer, one capacity check per number.
#include <bits/stdc++.h>
static char obuf[1 << 16]; static size_t on = 0;
static void flush() { fwrite(obuf, 1, on, stdout); on = 0; }
static void put(long long x) {
  char b[24]; int i = 24;
  do { b[--i] = char('0' + x % 10); x /= 10; } while (x);
  if (on + 25 > sizeof obuf) flush();
  memcpy(obuf + on, b + i, 24 - i); on += 24 - i; obuf[on++] = '\n';
}
int main() { int n; if (scanf("%d", &n) != 1) return 1; for (int i = 0; i < n; i++) put(i % 2000 + 1); flush(); }
