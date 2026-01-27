import java.nio.file.*;

public class debug_path {
    public static void main(String[] args) {
        Path current = Paths.get("").toAbsolutePath();
        System.out.println("Current: " + current);
        Path parent1 = current.getParent();
        Path parent2 = parent1.getParent();
        Path parent3 = parent2.getParent();
        System.out.println("Parent 3: " + parent3);
        Path testDocs = parent3.resolve("test_documents");
        System.out.println("Test docs: " + testDocs);
        System.out.println("Exists: " + Files.exists(testDocs));

        // Check a specific doc
        Path pdfFile = testDocs.resolve("pdfs/fake_memo.pdf");
        System.out.println("PDF file: " + pdfFile);
        System.out.println("PDF exists: " + Files.exists(pdfFile));
    }
}
