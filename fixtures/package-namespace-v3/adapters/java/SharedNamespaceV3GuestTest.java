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
    @Test void pinnedScalarSessionGuestAndSelectiveChecks()throws Exception{
        Path root=fixture();
        var providerManifest=Files.readString(root.resolve("checkout/.ores.pkg.toml"));
        var sources=new TreeMap<String,String>();
        try(var files=Files.list(root.resolve("checkout/src"))){
            for(Path p:files.toList())sources.put("src/"+p.getFileName(),Files.readString(p));
        }
        var client=new dev.oreslang.runtime.PinnedScalarNamespaceSession();
        var observer=new dev.oreslang.runtime.PinnedScalarNamespaceSession();
        String cm=Files.readString(root.resolve("client/.ores.pkg.toml")),om=Files.readString(root.resolve("observer/.ores.pkg.toml"));
        String cs=Files.readString(root.resolve("client/src/main.ores")),os=Files.readString(root.resolve("observer/src/main.ores"));
        var out=new ByteArrayOutputStream();var err=new ByteArrayOutputStream();
        var cold=client.run(cs,cm,providerManifest,sources,out,err);assertEquals(List.of("0","1","2","3"),out.toString(StandardCharsets.UTF_8).lines().toList());
        assertEquals(5,cold.build().rebuiltUnits().size());assertEquals(12,cold.inventoryParsed());assertEquals(0,cold.inventoryReused());assertEquals(1,cold.consumerParsed());out.reset();
        var other=observer.run(os,om,providerManifest,sources,out,err);assertEquals(List.of("4","5","6"),out.toString(StandardCharsets.UTF_8).lines().toList());assertEquals(4,other.build().rebuiltUnits().size());out.reset();
        var warm=client.run(cs,cm,providerManifest,sources,out,err);assertEquals(List.of("0","1","2","3"),out.toString(StandardCharsets.UTF_8).lines().toList());assertTrue(warm.build().rebuiltUnits().isEmpty());assertTrue(warm.build().reparsedUnits().isEmpty());assertEquals(0,warm.inventoryParsed());assertEquals(12,warm.inventoryReused());assertEquals(0,warm.consumerParsed());out.reset();
        var changed=new TreeMap<>(sources);changed.put("src/unit0.ores",changed.get("src/unit0.ores").replace("return 0","return 42"));
        var stale=assertThrows(IllegalArgumentException.class,()->client.run(cs,cm,providerManifest,changed,out,err));assertTrue(stale.getMessage().contains("source pin mismatch"));assertEquals(0,out.size());
        String old="8e5170a12e8950f934538ae7055d112a147361903f330b8632eaa71df5a2ebae";
        String edit="85f4245442b6854150a0e8247361287e69d6d355f1aeb8383f25373a0ab77f2e";
        String editedManifest=cm.replace(old,edit);
        var edited=client.run(cs,editedManifest,providerManifest,changed,out,err);assertEquals(List.of("42","1","2","3"),out.toString(StandardCharsets.UTF_8).lines().toList());
        assertEquals(Set.of("provider/src/unit0.ores"),edited.build().rebuiltUnits());assertEquals(Set.of("provider/src/unit0.ores"),edited.build().reparsedUnits());assertEquals(1,edited.inventoryParsed());assertEquals(11,edited.inventoryReused());assertEquals(0,edited.consumerParsed());out.reset();
        var unaffected=observer.run(os,om.replace(old,edit),providerManifest,changed,out,err);
        assertEquals(List.of("4","5","6"),out.toString(StandardCharsets.UTF_8).lines().toList());assertTrue(unaffected.build().rebuiltUnits().isEmpty());assertEquals(1,unaffected.inventoryParsed());assertEquals(11,unaffected.inventoryReused());out.reset();
        var clean=new dev.oreslang.runtime.PinnedScalarNamespaceSession();
        clean.run(cs,editedManifest,providerManifest,changed,out,err);assertEquals(List.of("42","1","2","3"),out.toString(StandardCharsets.UTF_8).lines().toList());out.reset();
        var abi=new TreeMap<>(changed);abi.put("src/unit0.ores","namespace billing;\npub fnc function0(int x): int { return x; }\n");
        String abiManifest=cm.replace(old,"dbe6a14b22fba63b297c140b7b6b0f0e49e6a46b5646fc8d23682fd0c66ef069");
        var bad=assertThrows(IllegalArgumentException.class,()->client.run(cs,abiManifest,providerManifest,abi,out,err));assertTrue(bad.getMessage().startsWith("E_NAMESPACE_ARITY:"),bad.getMessage());assertEquals(0,out.size());
        var recovered=client.run(cs,editedManifest,providerManifest,changed,out,err);assertTrue(recovered.build().rebuiltUnits().isEmpty());assertEquals(0,recovered.inventoryParsed());assertEquals(12,recovered.inventoryReused());
        assertEquals(List.of("42","1","2","3"),out.toString(StandardCharsets.UTF_8).lines().toList());
        System.out.println("V3_JAVA_SCALAR_SESSION|guest=0/1/2/3->42/1/2/3|observer=4/5/6|checked=5/0/1|observer-edit=0|clean-differential|stale-pin|abi-denial|rollback|inventory-parse=12/0/1|consumer-parse=1/0/0|fresh-runtime");
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
    @Test void launcherValidateAndRejectUnsafeFilePlan()throws Exception{
        Path root=fixture(),entry=root.resolve("client/src/main.ores");
        var build=LinkedProgramRunner.validate(entry,Map.of());
        assertEquals(5,build.units().size());
        assertTrue(build.units().containsKey(entry.toAbsolutePath().toString()));
        assertTrue(build.units().containsKey(root.resolve("checkout/src/unit0.ores").toAbsolutePath().toString()));
        var out=new ByteArrayOutputStream();
        var denied=new IsolatePolicy(Set.of(),64L*1024*1024,64,java.time.Duration.ofSeconds(10));
        assertThrows(Exception.class,()->LinkedProgramRunner.run(entry,denied,ExecutionProfile.serverJit(),Set.of(),Map.of(),out,new ByteArrayOutputStream()));
        assertEquals(0,out.size());
        var provider=root.resolve("checkout/src/unit0.ores"),saved=root.resolve("checkout/src/unit0.saved");
        Files.move(provider,saved);Files.createSymbolicLink(provider,saved);
        var symlink=assertThrows(Exception.class,()->run(root,"client"));
        assertTrue(symlink.getMessage().contains("symlinked snapshot path"),symlink.getMessage());
        Files.delete(provider);Files.move(saved,provider);
        var manifest=root.resolve("client/.ores.pkg.toml");String original=Files.readString(manifest);
        Files.writeString(manifest,original.replace("../checkout/.ores.pkg.toml","../../checkout/.ores.pkg.toml"));
        assertThrows(Exception.class,()->run(root,"client"));Files.writeString(manifest,original);
        Files.writeString(root.resolve("client/.atto.pkg.toml"),"schema_version = \"1\"\n");
        var ambiguous=assertThrows(Exception.class,()->run(root,"client"));assertTrue(ambiguous.getMessage().contains("ambiguous"));
        System.out.println("V3_JAVA_FILE_PLAN|physical-ids|validate|stdout-denied|symlink-denied|traversal-denied|ambiguous-denied");
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
