# AttoLang real Java suspends/await funded mirror

Source PR head: 40121bf30f419ce2a7480d3cd644a75cbc6e4311
Source full Git tree (not reproduced here): 819bf10d9186193b6f84c90a3e3ae18790d32a6c
Funded repo donor branch head: 73bcf12b013b0b36a2032c5d79a848dd4d54a53a

Exactly 268 pinned source blobs: 258 Java production files, 3 main resource files, pom.xml, 3 focused JUnit tests and 3 canonical .atto fixtures. Every file is verified at CI runtime with git hash-object against ci/suspends-exact-blobs.manifest.

This is an exact compiler-production-subtree and focused-test overlay, NOT a whole-repo tree-identity proof. Source repo's other JUnit tests, most scripts and docs are absent; never claim full Maven test suite or full source tree parity.

The source repo stays private and is not checked out across orgs; no secrets or cross-org credentials are used. The genuine GraalVM25 compile plus JUnit guest tests are the intended evidence. A zero-step job or a host-only test is not a guest pass.
