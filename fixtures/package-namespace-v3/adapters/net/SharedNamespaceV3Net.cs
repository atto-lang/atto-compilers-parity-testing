// Explicit fixture adapter: .NET registry currently accepts one source per
// namespace. Merge the authenticated scalar declaration texts, not results.
// The two unnamespaced files are authenticated but not exported.
// NativeAOT hosts execute the actual generated CIL with encoded return values.
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using Ores.Net.Compiler;

internal static class SharedNamespaceV3Net
{
    const string Name="@org/project/checkout",Pin="8e5170a12e8950f934538ae7055d112a147361903f330b8632eaa71df5a2ebae",Edit="85f4245442b6854150a0e8247361287e69d6d355f1aeb8383f25373a0ab77f2e",Abi="dbe6a14b22fba63b297c140b7b6b0f0e49e6a46b5646fc8d23682fd0c66ef069";
    static byte[] Bytes(string s)=>new UTF8Encoding(false,true).GetBytes(s);
    static string Sha(byte[] b)=>Convert.ToHexString(SHA256.HashData(b)).ToLowerInvariant();
    static void Check(bool yes,string detail){if(!yes)throw new InvalidOperationException(detail);}
    static string Digest(SortedDictionary<string,string> files)=>Sha(Bytes(string.Concat(files.Select(x=>x.Key+"\n"+Sha(Bytes(x.Value))+"\n"))));
    static void Deny(string code,Action f){try{f();}catch(FormatException e) when(e.Message.StartsWith(code+":",StringComparison.Ordinal)){return;}throw new InvalidOperationException("expected "+code);}
    static (byte[] manifest,byte[] locked,Dictionary<string,byte[]> sources) Adapt(SortedDictionary<string,string> files){
        var ns=new SortedDictionary<string,StringBuilder>(StringComparer.Ordinal);
        foreach(var (path,source) in files){
            if(!source.StartsWith("namespace ",StringComparison.Ordinal)){Check(path.StartsWith("src/local",StringComparison.Ordinal),"unexpected root unit");continue;}
            int semi=source.IndexOf(';');string name=source[10..semi];Check(name is "billing" or "identity" or "telemetry","unknown namespace");
            if(!ns.TryGetValue(name,out var b)){b=new StringBuilder("use namespace "+name+";\n");ns.Add(name,b);}
            b.Append(source[(semi+1)..]);
        }
        var manifest=new StringBuilder("[package]\nname=\""+Name+"\"\nversion=\"1.0.0\"\n[namespaces]\n");
        var sources=new Dictionary<string,byte[]>(StringComparer.Ordinal);
        foreach(var (name,body) in ns){string path="src/"+name+".atto";manifest.Append(name+"=\""+path+"\"\n");sources.Add(path,Bytes(body.ToString()));}
        var mb=Bytes(manifest.ToString());var l=new StringBuilder("[lock]\nmanifest_sha256=\""+Sha(mb)+"\"\n[files]\n");
        foreach(var (p,b) in sources)l.Append("\""+p+"\"=\""+Sha(b)+"\"\n");
        return(mb,Bytes(l.ToString()),sources);
    }
    static FrozenImportSnapshot Verify((byte[] manifest,byte[] locked,Dictionary<string,byte[]> sources) s)
        =>FrozenImportSnapshot.Verify(Name,"1.0.0",Sha(s.locked),s.manifest,s.locked,s.sources);
    internal static void Run(string root,string output){
        var files=new SortedDictionary<string,string>(StringComparer.Ordinal);
        foreach(var file in Directory.GetFiles(Path.Combine(root,"checkout/src"))){Check((File.GetAttributes(file)&FileAttributes.ReparsePoint)==0,"fixture symlink");files.Add("src/"+Path.GetFileName(file),File.ReadAllText(file));}
        Check(files.Count==12&&Digest(files)==Pin,"original inventory pin");
        Check(Sha(File.ReadAllBytes(Path.Combine(root,"checkout/.ores.pkg.toml")))=="72721b058d5367d74ed1bc4694e79ad13742db24513a192fb25851a3d391912f","manifest pin");
        string Consumer(string client,string ns,int start,int end){
            string m=File.ReadAllText(Path.Combine(root,client,".ores.pkg.toml"));Check(m.Contains(Pin,StringComparison.Ordinal),"consumer source pin");
            var expected=new StringBuilder("namespace app;\nimport namespace "+ns+" as provider from \"checkout\";\npub fnc main(): void {\n");string expr="0";
            for(int i=start;i<end;i++){expected.Append("  stdio.println(provider.function"+i+"());\n");expr="("+expr+")*100+provider."+ns+".function"+i+"()";}
            expected.Append("  return;\n}\n");Check(File.ReadAllText(Path.Combine(root,client,"src/main.ores"))==expected.ToString(),"consumer grammar");
            return "import pkg '"+Name+"' as provider; pub fnc main()->int{return "+expr+";}";
        }
        string billing=Consumer("client","billing",0,4),observer=Consumer("observer","identity",4,7);
        Directory.CreateDirectory(output);
        var original=Adapt(files);var p=Verify(original);
        var clientSession=new FrozenCilSession();var observerSession=new FrozenCilSession();
        var semantic=new OresStructuredParser.FrozenScalarSession();
        var semCold=semantic.Compile(billing,new[]{p});Check(semCold.ReparsedProviders.Count==3&&semCold.ReusedProviders.Count==0,"cold parser counts");
        var semWarm=semantic.Compile(billing,new[]{p});Check(semWarm.ReparsedProviders.Count==0&&semWarm.ReusedProviders.Count==3,"warm parser counts");
        byte[] Clean(string consumer,FrozenImportSnapshot provider)=>CilEmitter.BuildLibrary(OresStructuredParser.CompileFrozenScalar(consumer,new[]{provider}));
        void Write(string lane,byte[] b)=>File.WriteAllBytes(Path.Combine(output,lane+".dll"),b);
        var cold=clientSession.Compile(billing,new[]{p},true);Check(!cold.Reused&&cold.Cil.AsSpan().SequenceEqual(Clean(billing,p)),"cold clean");Write("cold",cold.Cil);
        var warm=clientSession.Compile(billing,new[]{p});Check(warm.Reused&&warm.Cil.AsSpan().SequenceEqual(cold.Cil),"warm cache");Write("warm",warm.Cil);
        var ob=observerSession.Compile(observer,new[]{p});Write("observer",ob.Cil);
        files["src/unit0.ores"]=files["src/unit0.ores"].Replace("return 0","return 42",StringComparison.Ordinal);Check(Digest(files)==Edit,"edit pin");
        var changed=Adapt(files);Deny("E_LOCK_HASH",()=>FrozenImportSnapshot.Verify(Name,"1.0.0",Sha(original.locked),original.manifest,original.locked,changed.sources));
        var p2=Verify(changed);
        var semEdit=semantic.Compile(billing,new[]{p2});Check(semEdit.ReparsedProviders.Count==1&&semEdit.ReusedProviders.Count==2,"repinned namespace parser counts");
        Check(CilEmitter.BuildLibrary(semEdit.Module).AsSpan().SequenceEqual(Clean(billing,p2)),"semantic clean differential");
        var semOther=semantic.Compile(observer,new[]{p2});Check(semOther.ReparsedProviders.Count==0&&semOther.ReusedProviders.Count==3,"observer parse counts");
        var edited=clientSession.Compile(billing,new[]{p2});Check(!edited.Reused&&edited.Cil.AsSpan().SequenceEqual(Clean(billing,p2)),"edit differential");Write("edit",edited.Cil);
        var ob2=observerSession.Compile(observer,new[]{p2});Check(ob2.Cil.AsSpan().SequenceEqual(Clean(observer,p2)),"observer differential");Write("observer-edit",ob2.Cil);
        Console.WriteLine("V3_NET_OBSERVER_ARTIFACT_CHANGED="+!ob.Cil.AsSpan().SequenceEqual(ob2.Cil));
        files["src/unit0.ores"]="namespace billing;\npub fnc function0(int x): int { return x; }\n";Check(Digest(files)==Abi,"ABI pin");
        var bad=Verify(Adapt(files));
        try{clientSession.Compile(billing,new[]{bad});throw new InvalidOperationException("ABI admitted");}
        catch(FormatException e){Check(e.Message.StartsWith("E_IMPORT_KIND: selected function must accept exact signed-int arguments and return int",StringComparison.Ordinal)||e.Message.StartsWith("E_ARITY:",StringComparison.Ordinal),"wrong ABI rejection "+e.Message);Console.WriteLine("V3_NET_ABI_REJECT|"+e.Message);}
        try{semantic.Compile(billing,new[]{bad});throw new InvalidOperationException("semantic ABI admitted");}catch(FormatException e){Check(e.Message.StartsWith("E_IMPORT_KIND: selected function must accept exact signed-int arguments and return int",StringComparison.Ordinal)||e.Message.StartsWith("E_ARITY:",StringComparison.Ordinal),"wrong semantic ABI rejection "+e.Message);}
        var semRecovery=semantic.Compile(billing,new[]{p2});Check(semRecovery.ReparsedProviders.Count==0&&semRecovery.ReusedProviders.Count==3,"semantic rollback");
        Console.WriteLine("V3_NET_NAMESPACE_PARSE|cold=3/0|warm=0/3|edit=1/2|observer=0/3|rollback=0/3");
        var recovered=clientSession.Compile(billing,new[]{p2});Check(recovered.Reused&&recovered.Cil.AsSpan().SequenceEqual(edited.Cil),"rollback");
        Console.WriteLine("V3_NET_CIL|cold-warm|clean-differential|stale-pin|abi-denied|rollback|adapted-one-file-per-namespace");
    }
}
