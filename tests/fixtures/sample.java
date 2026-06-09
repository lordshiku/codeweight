// Sample Java class for codeweight integration tests.

public class Sample {
    public int add(int a, int b) {
        if (a < 0) {
            return b;
        }
        return a + b;
    }

    public String classify(int x) {
        if (x > 10) {
            return "big";
        } else if (x > 5) {
            return "mid";
        } else {
            return "small";
        }
    }
}
