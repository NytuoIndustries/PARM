package shield;

public class Main {
    public static void main(String[] args) {
        Avengers avengers = new Avengers();
        System.out.println("AVENGERS! ASSEMBLE!");
        avengers.assemble(args);
        System.out.println("Avengers have assembled!");
    }
}