class Tiny {
    private static int add(int a, int b) {
        return a + b;
    }

    private static String test() {
        return null;
    }

    private static int get_number() {
        return 36;
    }

    public static void main(String[] argv) {
        for (int i = 0; i < 10000; i++) {
            int x = get_number();
            int y = 6;

            int z = add(x, y);
        }
    }
}
