//package Mock_CCC_Senior21;

import java.util.*;
import java.io.*;

public class Problem1 {
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
		new Problem1().solve(in, pw);
		pw.close();
	}

	public void solve(Input in, PrintWriter pw) throws Exception {
		char[] al = in.next().toCharArray();
		char[] arr = in.next().toCharArray();
		int n = arr.length;
		TreeSet<String> ans = new TreeSet<>();
		//change
		for(int i = 0; i<n; i++) {
			for(char c: al) {
				char[] cur = arr.clone();
				cur[i] = c;
				ans.add(new String(cur));
			}
		}
		//remove
		for(int i = 0; i<n; i++) {
			char[] next = new char[n-1];
			System.arraycopy(arr, 0, next, 0, i);
			System.arraycopy(arr, i+1, next, i, n-i-1);
			ans.add(new String(next));
		}
		//add
		for(int i = 0; i<n; i++) {
			char[] next = new char[n+1];
			System.arraycopy(arr, 0, next, 0, i);
			System.arraycopy(arr, i, next, i+1, n-i);
			for(char c: al) {
				next[i] = c;
				ans.add(new String(next));
			}
		}
		for(String s: ans) {
			pw.println(s);
		}
	}
}