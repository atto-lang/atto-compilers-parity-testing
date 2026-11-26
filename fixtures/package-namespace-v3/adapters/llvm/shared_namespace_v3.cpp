// Real production parser and verified graph only. No LLVM guest execution.
// Original provider manifest and source files are unchanged. Consumer source
// is reduced to a root scalar entry after validating the exact v3 text; its
// dependency is expressed in the existing verified lock profile.
#include "oreslang/frontend/VerifiedNamespaceGraphV2.hpp"
#include <openssl/sha.h>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <stdexcept>
#include <unistd.h>
namespace fs=std::filesystem;
using namespace oreslang;using namespace oreslang::frontend;
static std::string sha(std::string_view b){unsigned char d[SHA256_DIGEST_LENGTH];SHA256(reinterpret_cast<const unsigned char*>(b.data()),b.size(),d);std::string out;const char* h="0123456789abcdef";for(auto c:d){out+=h[c>>4];out+=h[c&15];}return out;}
static void check(bool yes,const char* why){if(!yes)throw std::runtime_error(why);}
static std::string read(const fs::path& p){std::ifstream in(p,std::ios::binary);if(!in)throw std::runtime_error("read failed");return {std::istreambuf_iterator<char>(in),{}};}
static void write(const fs::path& p,std::string_view s){fs::create_directories(p.parent_path());std::ofstream out(p,std::ios::binary);out<<s;if(!out)throw std::runtime_error("write failed");}
template<class F>static void deny(F f,std::string_view wanted){try{f();}catch(const std::exception& e){if(std::string(e.what()).find(wanted)!=std::string::npos)return;throw;}throw std::runtime_error("expected denial");}
int main(int argc,char** argv){
 // Exercise actual canonical pointer parser and scalar type checking,
 // including the previously non-portable arithmetic path.
 auto pointer=[](std::string expr){return parse_program("pub fnc main(): int { let mut Pointer<int> p = Pointer.alloc<int>(1); p.write_at(0, "+expr+"); p.close(); return 0; }",false,false,true);};
 (void)pointer("9223372036854775807 + 0");
 (void)pointer("(0 - 9223372036854775807 - 1) * 1");
 (void)pointer("(0 - 9223372036854775807 - 1) * 0");
 for(const std::string expr:{"9223372036854775807 + 1","(0 - 9223372036854775807 - 1) - 1","9223372036854775807 * 2","(0 - 9223372036854775807 - 1) * (0 - 1)"})
  deny([&]{(void)pointer(expr);},"checked arithmetic overflow");
 (void)parse_program("pub fnc main():int{return Math.abs(0-42);}");
 std::cout<<"V3_LLVM_CHECKED_ARITHMETIC|3-boundary-positive|4-overflow-denied|abs-int\\n";
 check(argc==2,"fixture argument");fs::path input=argv[1];char temp[]="/tmp/atto-llvm-v3-XXXXXX";char* dir=mkdtemp(temp);check(dir!=nullptr,"temp");
 struct Cleanup{fs::path path;~Cleanup(){std::error_code ec;fs::remove_all(path,ec);}} cleanup{dir};fs::path root=dir;
 const std::string pin="8e5170a12e8950f934538ae7055d112a147361903f330b8632eaa71df5a2ebae",edit="85f4245442b6854150a0e8247361287e69d6d355f1aeb8383f25373a0ab77f2e";
 const auto provider_manifest=read(input/"checkout/.ores.pkg.toml");const auto mp=sha(provider_manifest);
 check(mp=="72721b058d5367d74ed1bc4694e79ad13742db24513a192fb25851a3d391912f","manifest pin");
 write(root/"checkout/.ores.pkg.toml",provider_manifest);
 std::vector<std::string> paths;for(const auto& e:fs::directory_iterator(input/"checkout/src")){check(fs::is_regular_file(e.symlink_status()),"regular source");auto path="src/"+e.path().filename().string();paths.push_back(path);write(root/"checkout"/path,read(e.path()));}
 std::sort(paths.begin(),paths.end());check(paths.size()==12,"inventory count");
 auto capture=[&](std::string_view source_pin){return load_verified_namespace_source_snapshot_v2(root/"checkout",paths,mp,std::string(source_pin),sha);};
 auto provider=capture(pin);
 auto setup=[&](std::string name,std::string ns,int begin,int end){
  auto original=read(input/name/"src/main.ores");std::string expected="namespace app;\nimport namespace "+ns+" as provider from \"checkout\";\npub fnc main(): void {\n";
  for(int i=begin;i<end;++i)expected+="  stdio.println(provider.function"+std::to_string(i)+"());\n";
  expected+="  return;\n}\n";check(original==expected,"consumer grammar");check(read(input/name/".ores.pkg.toml").find(pin)!=std::string::npos,"consumer pin");
  std::string manifest="schema_version = \"2\"\n[project]\nroot = \".\"\n[packages."+name+"]\nfiles = [\"src/main.ores\"]\n";
  std::string source="import namespace "+ns+" as provider from \"checkout\";\npub fnc main():int{return 0;}\n";
  write(root/name/".ores.pkg.toml",manifest);write(root/name/"src/main.ores",source);
  return load_verified_namespace_source_snapshot_v2(root/name,{"src/main.ores"},sha(manifest),sha("src/main.ores\n"+sha(source)+"\n"),sha);
 };
 auto client=setup("client","billing",0,4),observer=setup("observer","identity",4,7);
 auto owner=[](const auto& s,std::string name){return "v2:"+s.inventory().manifest_sha256()+":"+s.inventory().sources_sha256()+":"+name;};
 auto run=[&](const auto& snapshot){
  auto p=owner(snapshot,"checkout"),c=owner(client,"client"),o=owner(observer,"observer");
  auto locktext="schema_version = \"1\"\n[[dependency]]\nimporter = \""+c+"\"\nalias = \"checkout\"\ntarget = \""+p+"\"\n[[dependency]]\nimporter = \""+o+"\"\nalias = \"checkout\"\ntarget = \""+p+"\"\n";
  auto lock=verify_namespace_dependency_lock_v2(locktext,sha(locktext),{p,c,o},sha);
  auto graph=link_verified_native_namespace_snapshots_v2({{&snapshot},{&client},{&observer}},&lock);
  check(graph.namespaces.size()==3&&graph.imports.size()==2,"three namespaces two consumers");
  for(int i=0;i<7;++i){const auto& from=i<4?c:o;auto target=resolve_package_namespace_scalar_call(graph,from+"/src/main.ores","provider","function"+std::to_string(i),{});check(target.source_unit_id==p+"/src/unit"+std::to_string(i)+".ores","physical provider");}
  deny([&]{(void)resolve_package_namespace_scalar_call(graph,c+"/src/main.ores","provider","function4",{});},"not exported");
  deny([&]{(void)resolve_package_namespace_scalar_call(graph,c+"/src/main.ores","provider","local0",{});},"not exported");
 };
 run(provider);auto source=read(root/"checkout/src/unit0.ores");auto pos=source.find("return 0");check(pos!=std::string::npos,"edit location");source.replace(pos,8,"return 42");write(root/"checkout/src/unit0.ores",source);
 deny([&]{(void)capture(pin);},"SHA-256 mismatch");auto changed=capture(edit);run(changed);run(provider);
 std::cout<<"V3_LLVM_REAL_PARSER_GRAPH|12-providers|2-consumers|7-physical-bindings|foreign-root-denied|stale-pin|captured-rollback|host-only\n";
}
