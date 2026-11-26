package dev.oreslang;

import dev.oreslang.runtime.ExecutionProfile;
import dev.oreslang.runtime.IsolatePolicy;
import dev.oreslang.runtime.LinkedProgramRunner;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import java.io.ByteArrayOutputStream;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import static org.junit.jupiter.api.Assertions.*;

/** Exact shared fixture adapter. This runs the actual compiler/linker and
 * guest, not a source oracle. No incremental cache counters are implied:
 * the runner currently creates a fresh compiler per invocation. */
final class SharedNamespaceV3GuestTest {
    @TempDir Path work;
    private Path fixture()throws Exception{
        Path source=Path.of(Objects.requireNonNull(System.getenv("ATTO_V3_FIXTURE")));
        try(var walk=Files.walk(source)){
            for(Path p:walk.toList()){
                Path dest=work.resolve(source.relativize(p));
                if(Files.isDirectory(p,LinkOption.NOFOLLOW_LINKS))Files.createDirectories(dest);
                else if(Files.isRegularFile(p,LinkOption.NOFOLLOW_LINKS))Files.copy(p,dest);
                else throw new IllegalArgumentException("fixture nonregular entry");
            }
        }
        return work;
    }
    private List<String> run(Path root,String consumer)throws Exception{
        var out=new ByteArrayOutputStream();
        LinkedProgramRunner.run(root.resolve(consumer+"/src/main.ores"),IsolatePolicy.developer(),
            ExecutionProfile.serverJit(),Set.of(),Map.of(),out,new ByteArrayOutputStream());
        return out.toString(StandardCharsets.UTF_8).lines().toList();
    }
    @Test void exactPinnedInventoryAdmission()throws Exception{
        Path root=fixture();
        var providerManifest=Files.readString(root.resolve("checkout/.ores.pkg.toml"));
        var sources=new TreeMap<String,String>();
        try(var files=Files.list(root.resolve("checkout/src"))){
            for(Path p:files.toList())sources.put("src/"+p.getFileName(),Files.readString(p));
        }
        assertEquals(12,sources.size());
        for(String consumer:List.of("client","observer")){
            var program=dev.oreslang.parser.Parser.parse(Files.readString(root.resolve(consumer+"/src/main.ores")));
            var manifest=Files.readString(root.resolve(consumer+"/.ores.pkg.toml"));
            var selected=dev.oreslang.compiler.PinnedNamespaceInventory.admit(program.imports().getFirst(),manifest,providerManifest,sources);
            assertEquals(consumer.equals("client")?4:3,selected.providers().size());
            assertEquals(consumer.equals("client")?"billing":"identity",selected.namespace());
            var changed=new TreeMap<>(sources);changed.put("src/unit0.ores",changed.get("src/unit0.ores").replace("return 0","return 42"));
            var stale=assertThrows(IllegalArgumentException.class,()->dev.oreslang.compiler.PinnedNamespaceInventory.admit(program.imports().getFirst(),manifest,providerManifest,changed));
            assertTrue(stale.getMessage().contains("source pin mismatch"),stale.getMessage());
            var updated=manifest.replace("8e5170a12e8950f934538ae7055d112a147361903f330b8632eaa71df5a2ebae","85f4245442b6854150a0e8247361287e69d6d355f1aeb8383f25373a0ab77f2e");
            assertEquals(selected.providers().size(),dev.oreslang.compiler.PinnedNamespaceInventory.admit(program.imports().getFirst(),updated,providerManifest,changed).providers().size());
        }
        System.out.println("V3_JAVA_INVENTORY|12-providers|two-consumers|4/3-selection|stale-pin-denied|repinned-edit-admitted|not-linked");
    }
    @Test void actualColdBodyEditAndIndependentConsumer()throws Exception{
        Path root=fixture();
        assertEquals(List.of("0","1","2","3"),run(root,"client"));
        assertEquals(List.of("4","5","6"),run(root,"observer"));
        var provider=root.resolve("checkout/src/unit0.ores");
        Files.writeString(provider,Files.readString(provider).replace("return 0","return 42"));
        for(String consumer:List.of("client","observer")){
            var manifest=root.resolve(consumer+"/.ores.pkg.toml");
            Files.writeString(manifest,Files.readString(manifest).replace(
                "8e5170a12e8950f934538ae7055d112a147361903f330b8632eaa71df5a2ebae",
                "85f4245442b6854150a0e8247361287e69d6d355f1aeb8383f25373a0ab77f2e"));
        }
        assertEquals(List.of("42","1","2","3"),run(root,"client"));
        assertEquals(List.of("4","5","6"),run(root,"observer"));
        System.out.println("V3_JAVA_GUEST|cold=0/1/2/3|edited=42/1/2/3|observer=4/5/6|clean-only");
    }
    @Test void stalePinRejectsBeforeGuest()throws Exception{
        Path root=fixture();var provider=root.resolve("checkout/src/unit0.ores");
        // Baseline success is mandatory: a generic unsupported failure is
        // not a stale-pin rejection.
        assertEquals(List.of("0","1","2","3"),run(root,"client"));
        Files.writeString(provider,Files.readString(provider).replace("return 0","return 42"));
        Exception e=assertThrows(Exception.class,()->run(root,"client"));
        assertTrue(e.getMessage()!=null&&e.getMessage().toLowerCase(Locale.ROOT).matches("(?s).*(pin|digest|sha256|hash).*"),String.valueOf(e));
        System.out.println("V3_JAVA_STALE_PIN_DENIED");
    }
}
