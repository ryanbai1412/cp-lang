import java.util.*;
import java.io.*;
import java.math.*;

public class Main {
	public static void main(String[] args) {
		System.out.println(new Scanner(System.in).nextBigInteger().subtract(BigInteger.ONE).nextProbablePrime());
	}
}