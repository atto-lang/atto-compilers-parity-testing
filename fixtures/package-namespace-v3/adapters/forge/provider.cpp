#include "compiler/namespace_provider.hpp"
#include <iostream>
#include <iterator>
int main(int argc,char** argv){
  if(argc!=3)return 2;
  try{
    std::string s;
    for(char c;std::cin.get(c);){if(s.size()==65536)throw std::runtime_error("E_NAMESPACE_PROFILE: source budget");s.push_back(c);}
    std::cout<<oresforge::compile_namespace_provider(s,argv[1],argv[2],oresforge::Arch::X86_64);
    return std::cout?0:3;
  }catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}
}
