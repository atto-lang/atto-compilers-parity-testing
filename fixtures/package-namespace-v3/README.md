# Pinned incremental namespace fixture v3

Synthetic source-only fixture; no compiler execution is implied by the integrity test.
Twelve provider source files (10 explicitly namespaced, 2 without namespace
headers) and two separate consumers (14 total source files). Keep v2 unchanged.
Both consumers pin the same complete provider inventory. The billing consumer
observes 0/1/2/3; the identity observer observes 4/5/6. The root-level local files
must not silently acquire an exported namespace or become importable through a
foreign namespace. This fixture uses existing v2 manifest and legacy .ores
compatibility syntax, not a new package grammar.

For each edit use a fresh copy, verify the initial tree and manifest membership,
apply the specified source change, and update both consumers' source pins.
A stale pin must reject before compile. On a body-only edit semantic validation
should recheck the owning unit, not the unrelated observer; relink the billing
consumer even when its typed metadata remains reusable. ABI change invalidates
the dependent consumer and must reject the old zero-argument call. A failed build
must preserve the prior accepted snapshot. No-op warm build must perform no
source transformation/type checking/codegen for unchanged units. Whole-artifact
cache hits alone do not establish these properties.

Record exact compiler head/tree, fixture tree, executed steps, clean vs
incremental output, actual reparse/recheck/codegen/link counters, rejection
diagnostics and rollback receipts. Unsupported or unexecuted is not a pass.
A compiler may conservatively rebuild more units, but that is a measured
incremental-parity gap, not full success. No synthetic oracle counts as compiler
execution.

## Consolidated restricted adapter candidates

`compiler-lanes.json` records exact candidate compiler heads and root trees for the
consolidated execution lanes. Existing Rust, Java, .NET and native graph adapters
remain intact. The added JS, Forge, Cranelift, HDL and LLVM-native adapters are
byte-identical to their separately reviewed candidate test files. The carrier
must rerun the combined fixture tree; historical receipts do not establish a
pass on this tree. No adapter may be relabeled as a default production linker.
