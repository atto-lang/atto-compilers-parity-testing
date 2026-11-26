#include "compiler/frontend.hpp"
#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <stdexcept>
#include <vector>
using namespace oresforge;namespace fs=std::filesystem;
static void check(bool b){if(!b)throw std::runtime_error("v3 assertion");}
static std::string read(fs::path p){std::ifstream in(p);check(bool(in));return {std::istreambuf_iterator<char>(in),{}};}
int main(int argc,char** argv){check(argc==2);fs::path root=argv[1];std::vector<IncrementalSession> sessions(7);std::vector<std::string> prior;
 for(std::string lane:{"cold","warm","edit"}){
  int hits=0;for(int i=0;i<7;i++){
   auto path=root/(lane=="warm"?"cold":lane)/("call"+std::to_string(i)+".atto");auto source=read(path);
   auto a=sessions[i].build(source,Arch::X86_64,lane=="cold");check(a.assembly==emit_runtime_asm(parse_runtime_source(source),Arch::X86_64));
   hits+=a.reused;if(lane=="cold")prior.push_back(a.assembly);else if(lane=="warm"||i!=0)check(a.assembly==prior[i]);
   std::ofstream out(root/(lane+"-"+std::to_string(i)+".s"));out<<a.assembly;check(bool(out));
  }check(hits==(lane=="cold"?0:lane=="warm"?7:6));std::cout<<"V3_FORGE_UNITS|"<<lane<<"|hits="<<hits<<"\n";
 }
 bool denied=false;try{sessions[0].build(read(root/"abi/call0.atto"),Arch::X86_64);}catch(const std::runtime_error&){denied=true;}check(denied);check(sessions[0].build(read(root/"edit/call0.atto"),Arch::X86_64).reused);
 std::cout<<"V3_FORGE_ABI_DENIAL_ROLLBACK\n";
}
