//package Mock_CCC_Senior21;

import java.util.*;
import java.io.*;

import static java.lang.Math.*;

public class Problem2 {
	static class Input {
		BufferedReader br = new BufferedReader(new InputStreamReader(System.in), 1<<16);
		StringTokenizer st = new StringTokenizer("");
		public String next() throws IOException {
			while(!st.hasMoreTokens()) {
				st = new StringTokenizer(br.readLine());
			}
			return st.nextToken();
		}
		public int nextInt() throws IOException {
			return Integer.parseInt(next());
		}
	}

	public static void main(String[] args) throws Exception {
		Input in = new Input();
		PrintWriter pw = new PrintWriter(System.out);
		new Problem2().solve(in, pw);
		pw.close();
	}

	final long mod = (long) 1e9+7;
	public void solve(Input in, PrintWriter pw) throws Exception {
		char[] arr = in.next().toCharArray();
		int[] cnt = new int[26];
		for(char c: arr) {
			cnt[c-'a']++;
		}
		long ans = 1;
		for(int i: cnt) {
			ans = (ans*(i+1))%mod;
		}
		pw.println(ans);
	}
}