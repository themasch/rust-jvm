class SimpleMath {
    private static int add(int a, int b) {
        return a + b;
    }

    private static int get_number() {
        return 36;
    }

    public static int testMe() {
        int a = get_number();
        int b = 10;

        return add(a, b);
    }

    public static void main(String[] argv) {
        int a = get_number();
        int b = 10;

        int c = add(a, b);
    }
}
