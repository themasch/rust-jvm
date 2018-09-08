class SimpleMathWithLoop {
    private static int add(int a, int b) {
        return a + b;
    }

    private static int get_number() {
        return 2;
    }

    public static int testMe() {
        int summe = 3;
        for (int i = 0; i < 100; i++) {
            int x = get_number();

            summe = add(x, summe);
        }

        return summe;
    }

    public static void main(String[] argv) {
        int summe = 3;
        for (int i = 0; i < 10; i++) {
            int x = get_number();

            summe = add(x, summe);
        }
    }
}
