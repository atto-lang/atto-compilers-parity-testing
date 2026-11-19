# Oreslang → AttoLang full-organization migration inventory

**Inventory checked:** 2026-11-19, via authenticated GitHub repository search. **Status:** discovery and mapping only; nothing in this file proves objects, branches, issues, or PRs have been migrated. Keep this document in a **private** Atto repository: most source repositories are private.

- Archived Oreslang repositories inventoried: **83**.
- Atto organization repositories inventoried: **73**.
- Source repositories with a plausible existing destination: **69**, mapping to **68 distinct** existing destination repositories. All are *candidate* equivalences until inspected.
- Sources needing destination decision/creation: **14**, including **2 ambiguous mapping(s)** and **12 without an existing destination**.
- Atto repositories without a clear mapping from this source inventory: `atto-playground`, `atto-borrow-checker.rs`, `atto-source.realtime`, `atto-automated-jobs`, `atto-skills.md`.

## Source → destination manifest (all 83)

Names are proposed matches, **not evidence of commit, branch, issue, or PR parity**. Rows marked "review" MUST NOT be automatically routed to a candidate destination.

| Archived source repository | Proposed Atto destination | Mapping status | Source visibility |
| --- | --- | --- | --- |
| [`oreslang-restful-server-perf-benchmarks`](https://github.com/ores-truffle-oreslang/oreslang-restful-server-perf-benchmarks) | [`atto-restful-server-perf-benchmarks`](https://github.com/atto-lang/atto-restful-server-perf-benchmarks) | Candidate (verify history/content) | Private |
| [`oreslang-static-analysis.rs`](https://github.com/ores-truffle-oreslang/oreslang-static-analysis.rs) | [`atto-static-analysis.rs`](https://github.com/atto-lang/atto-static-analysis.rs) | Candidate (verify history/content) | Private |
| [`oreslang-format`](https://github.com/ores-truffle-oreslang/oreslang-format) | [`atto-format`](https://github.com/atto-lang/atto-format) | Candidate (verify history/content) | Private |
| [`oreslang-ssr`](https://github.com/ores-truffle-oreslang/oreslang-ssr) | [`atto-ssr`](https://github.com/atto-lang/atto-ssr) | Candidate (verify history/content) | Private |
| [`oreslang-web-browser.ores`](https://github.com/ores-truffle-oreslang/oreslang-web-browser.ores) | [`atto-browser`](https://github.com/atto-lang/atto-browser) | Review ambiguous alias | Private |
| [`oreslang-platform.mobile`](https://github.com/ores-truffle-oreslang/oreslang-platform.mobile) | [`atto-platform.mobile`](https://github.com/atto-lang/atto-platform.mobile) | Candidate (verify history/content) | Private |
| [`oreslang-reverse-proxy-demo.ores`](https://github.com/ores-truffle-oreslang/oreslang-reverse-proxy-demo.ores) | [`atto-reverse-proxy-demo.atto`](https://github.com/atto-lang/atto-reverse-proxy-demo.atto) | Candidate (verify history/content) | Private |
| [`oreslang-spin`](https://github.com/ores-truffle-oreslang/oreslang-spin) | *No existing counterpart* | Missing destination | Private |
| [`oreslang-ai-agent-coordination-via-actors-demo`](https://github.com/ores-truffle-oreslang/oreslang-ai-agent-coordination-via-actors-demo) | [`atto-ai-agent-coordination-demo`](https://github.com/atto-lang/atto-ai-agent-coordination-demo) | Candidate (verify history/content) | Private |
| [`gh-linguist-fork`](https://github.com/ores-truffle-oreslang/gh-linguist-fork) | *No existing counterpart* | Missing destination | Public |
| [`rx-oreslang-channels`](https://github.com/ores-truffle-oreslang/rx-oreslang-channels) | [`atto-rx-channels`](https://github.com/atto-lang/atto-rx-channels) | Candidate (verify history/content) | Private |
| [`oreslang-platform.asic`](https://github.com/ores-truffle-oreslang/oreslang-platform.asic) | [`atto-platform.asic`](https://github.com/atto-lang/atto-platform.asic) | Candidate (verify history/content) | Private |
| [`ores-truffle-oreslang.github.io`](https://github.com/ores-truffle-oreslang/ores-truffle-oreslang.github.io) | [`atto-lang.github.io`](https://github.com/atto-lang/atto-lang.github.io) | Candidate (verify history/content) | Public |
| [`compiler-parity-website`](https://github.com/ores-truffle-oreslang/compiler-parity-website) | *No existing counterpart* | Missing destination | Private |
| [`attolang-language-spec.md`](https://github.com/ores-truffle-oreslang/attolang-language-spec.md) | [`atto-spec`](https://github.com/atto-lang/atto-spec) | Candidate (verify history/content) | Private |
| [`oreslang-rpc`](https://github.com/ores-truffle-oreslang/oreslang-rpc) | [`atto-rpc`](https://github.com/atto-lang/atto-rpc) | Candidate (verify history/content) | Private |
| [`oreslang-serialization-and-validation`](https://github.com/ores-truffle-oreslang/oreslang-serialization-and-validation) | [`atto-serialization-and-validation`](https://github.com/atto-lang/atto-serialization-and-validation) | Candidate (verify history/content) | Private |
| [`oreslang-durable-objects.ores`](https://github.com/ores-truffle-oreslang/oreslang-durable-objects.ores) | *No existing counterpart* | Missing destination | Private |
| [`oreslang-tensor`](https://github.com/ores-truffle-oreslang/oreslang-tensor) | [`atto-tensor`](https://github.com/atto-lang/atto-tensor) | Candidate (verify history/content) | Private |
| [`rx-ores`](https://github.com/ores-truffle-oreslang/rx-ores) | [`atto-rx`](https://github.com/atto-lang/atto-rx) | Candidate (verify history/content) | Private |
| [`vscode-plugins`](https://github.com/ores-truffle-oreslang/vscode-plugins) | [`atto-vscode`](https://github.com/atto-lang/atto-vscode) | Candidate (verify history/content) | Public |
| [`oreslang-common-source`](https://github.com/ores-truffle-oreslang/oreslang-common-source) | [`atto-common-source`](https://github.com/atto-lang/atto-common-source) | Candidate (verify history/content) | Private |
| [`oreslang-ml-conformance`](https://github.com/ores-truffle-oreslang/oreslang-ml-conformance) | [`atto-ml-conformance`](https://github.com/atto-lang/atto-ml-conformance) | Candidate (verify history/content) | Private |
| [`oreslang-orie-ieor`](https://github.com/ores-truffle-oreslang/oreslang-orie-ieor) | *No existing counterpart* | Missing destination | Private |
| [`rx-ores-tasks`](https://github.com/ores-truffle-oreslang/rx-ores-tasks) | [`atto-rx-tasks`](https://github.com/atto-lang/atto-rx-tasks) | Candidate (verify history/content) | Private |
| [`oreslang-source.js`](https://github.com/ores-truffle-oreslang/oreslang-source.js) | [`atto-source.js`](https://github.com/atto-lang/atto-source.js) | Candidate (verify history/content) | Private |
| [`oreslang-platform.embedded`](https://github.com/ores-truffle-oreslang/oreslang-platform.embedded) | [`atto-platform.embedded`](https://github.com/atto-lang/atto-platform.embedded) | Candidate (verify history/content) | Private |
| [`oreslang-demo-restful-server`](https://github.com/ores-truffle-oreslang/oreslang-demo-restful-server) | [`atto-demo-restful-server`](https://github.com/atto-lang/atto-demo-restful-server) | Candidate (verify history/content) | Public |
| [`oreslang-cli`](https://github.com/ores-truffle-oreslang/oreslang-cli) | [`atto-cli`](https://github.com/atto-lang/atto-cli) | Candidate (verify history/content) | Public |
| [`oreslang-runtime.realtime-java`](https://github.com/ores-truffle-oreslang/oreslang-runtime.realtime-java) | [`atto-runtime.realtime-java`](https://github.com/atto-lang/atto-runtime.realtime-java) | Candidate (verify history/content) | Private |
| [`oreslang-hpc-finance`](https://github.com/ores-truffle-oreslang/oreslang-hpc-finance) | *No existing counterpart* | Missing destination | Private |
| [`oreslang-data-science`](https://github.com/ores-truffle-oreslang/oreslang-data-science) | [`atto-data-science`](https://github.com/atto-lang/atto-data-science) | Candidate (verify history/content) | Private |
| [`oreslang-source.rs`](https://github.com/ores-truffle-oreslang/oreslang-source.rs) | [`atto-source.rs`](https://github.com/atto-lang/atto-source.rs) | Candidate (verify history/content) | Private |
| [`oreslang-websocket-server-demo.ores`](https://github.com/ores-truffle-oreslang/oreslang-websocket-server-demo.ores) | [`atto-websocket-server-demo.atto`](https://github.com/atto-lang/atto-websocket-server-demo.atto) | Candidate (verify history/content) | Private |
| [`oreslang-sg-stategraph.ores`](https://github.com/ores-truffle-oreslang/oreslang-sg-stategraph.ores) | [`atto-stategraph.atto`](https://github.com/atto-lang/atto-stategraph.atto) | Candidate (verify history/content) | Private |
| [`oreslang-phoenix-server.ores`](https://github.com/ores-truffle-oreslang/oreslang-phoenix-server.ores) | [`atto-phoenix-server.atto`](https://github.com/atto-lang/atto-phoenix-server.atto) | Candidate (verify history/content) | Private |
| [`oreslang-proof-of-concepts`](https://github.com/ores-truffle-oreslang/oreslang-proof-of-concepts) | [`atto-proof-of-concepts`](https://github.com/atto-lang/atto-proof-of-concepts) | Candidate (verify history/content) | Private |
| [`oreslang-test.cli`](https://github.com/ores-truffle-oreslang/oreslang-test.cli) | [`atto-test.cli`](https://github.com/atto-lang/atto-test.cli) | Candidate (verify history/content) | Private |
| [`oreslang-logic.ores`](https://github.com/ores-truffle-oreslang/oreslang-logic.ores) | [`atto-logic.atto`](https://github.com/atto-lang/atto-logic.atto) | Candidate (verify history/content) | Private |
| [`oreslang-language-specs.md`](https://github.com/ores-truffle-oreslang/oreslang-language-specs.md) | [`atto-spec`](https://github.com/atto-lang/atto-spec) | Candidate (verify history/content) | Private |
| [`oreslang-runtime-ui`](https://github.com/ores-truffle-oreslang/oreslang-runtime-ui) | [`atto-runtime-ui`](https://github.com/atto-lang/atto-runtime-ui) | Candidate (verify history/content) | Private |
| [`oreslang-target-profiles`](https://github.com/ores-truffle-oreslang/oreslang-target-profiles) | [`atto-target-profiles`](https://github.com/atto-lang/atto-target-profiles) | Candidate (verify history/content) | Private |
| [`rx-ores-callbacks`](https://github.com/ores-truffle-oreslang/rx-ores-callbacks) | [`atto-rx-callbacks`](https://github.com/atto-lang/atto-rx-callbacks) | Candidate (verify history/content) | Private |
| [`oreslang-linked-queue`](https://github.com/ores-truffle-oreslang/oreslang-linked-queue) | [`atto-linked-queue`](https://github.com/atto-lang/atto-linked-queue) | Candidate (verify history/content) | Private |
| [`oreslang-physics-demo.ores`](https://github.com/ores-truffle-oreslang/oreslang-physics-demo.ores) | [`atto-physics-demo.atto`](https://github.com/atto-lang/atto-physics-demo.atto) | Candidate (verify history/content) | Private |
| [`oreslang-db`](https://github.com/ores-truffle-oreslang/oreslang-db) | [`atto-db`](https://github.com/atto-lang/atto-db) | Candidate (verify history/content) | Private |
| [`oreslang-source.java`](https://github.com/ores-truffle-oreslang/oreslang-source.java) | [`atto-source.java`](https://github.com/atto-lang/atto-source.java) | Candidate (verify history/content) | Private |
| [`oreslang-source.net`](https://github.com/ores-truffle-oreslang/oreslang-source.net) | [`atto-source.net`](https://github.com/atto-lang/atto-source.net) | Candidate (verify history/content) | Private |
| [`oreslang-source.hdl`](https://github.com/ores-truffle-oreslang/oreslang-source.hdl) | [`atto-source.hdl`](https://github.com/atto-lang/atto-source.hdl) | Candidate (verify history/content) | Private |
| [`oreslang-source.forge`](https://github.com/ores-truffle-oreslang/oreslang-source.forge) | [`atto-source.forge`](https://github.com/atto-lang/atto-source.forge) | Candidate (verify history/content) | Private |
| [`oreslang-http-routing`](https://github.com/ores-truffle-oreslang/oreslang-http-routing) | [`atto-http-routing`](https://github.com/atto-lang/atto-http-routing) | Candidate (verify history/content) | Private |
| [`postgres-orm.ores`](https://github.com/ores-truffle-oreslang/postgres-orm.ores) | [`atto-postgres-orm.atto`](https://github.com/atto-lang/atto-postgres-orm.atto) | Candidate (verify history/content) | Private |
| [`oreslang-api-server.ores`](https://github.com/ores-truffle-oreslang/oreslang-api-server.ores) | [`atto-api-server.atto`](https://github.com/atto-lang/atto-api-server.atto) | Candidate (verify history/content) | Private |
| [`oreslang-compilers-parity-testing`](https://github.com/ores-truffle-oreslang/oreslang-compilers-parity-testing) | [`atto-compilers-parity-testing`](https://github.com/atto-lang/atto-compilers-parity-testing) | Candidate (verify history/content) | Private |
| [`ores-fm-mac-agent.ores`](https://github.com/ores-truffle-oreslang/ores-fm-mac-agent.ores) | *No existing counterpart* | Missing destination | Private |
| [`oreslang-demo`](https://github.com/ores-truffle-oreslang/oreslang-demo) | [`atto-demo`](https://github.com/atto-lang/atto-demo) | Candidate (verify history/content) | Private |
| [`atto-redis-clone.atto`](https://github.com/ores-truffle-oreslang/atto-redis-clone.atto) | *No existing counterpart* | Missing destination | Private |
| [`oreslang-platform.iot`](https://github.com/ores-truffle-oreslang/oreslang-platform.iot) | [`atto-platform.iot`](https://github.com/atto-lang/atto-platform.iot) | Candidate (verify history/content) | Private |
| [`oreslang-ml`](https://github.com/ores-truffle-oreslang/oreslang-ml) | [`atto-ml`](https://github.com/atto-lang/atto-ml) | Candidate (verify history/content) | Private |
| [`oreslang-design-patterns`](https://github.com/ores-truffle-oreslang/oreslang-design-patterns) | [`atto-design-patterns`](https://github.com/atto-lang/atto-design-patterns) | Candidate (verify history/content) | Private |
| [`oreslang-n8n-make-zapier-app`](https://github.com/ores-truffle-oreslang/oreslang-n8n-make-zapier-app) | [`atto-automated-jobs`](https://github.com/atto-lang/atto-automated-jobs) | Review ambiguous alias | Private |
| [`atto-source.atto`](https://github.com/ores-truffle-oreslang/atto-source.atto) | [`atto-source.atto`](https://github.com/atto-lang/atto-source.atto) | Candidate (verify history/content) | Private |
| [`oreslang-platform.desktop`](https://github.com/ores-truffle-oreslang/oreslang-platform.desktop) | [`atto-platform.desktop`](https://github.com/atto-lang/atto-platform.desktop) | Candidate (verify history/content) | Private |
| [`atto-ci`](https://github.com/ores-truffle-oreslang/atto-ci) | [`atto-ci`](https://github.com/atto-lang/atto-ci) | Candidate (verify history/content) | Private |
| [`oreslang-source.llvm`](https://github.com/ores-truffle-oreslang/oreslang-source.llvm) | [`atto-source.llvm`](https://github.com/atto-lang/atto-source.llvm) | Candidate (verify history/content) | Private |
| [`oreslana.ores`](https://github.com/ores-truffle-oreslang/oreslana.ores) | *No existing counterpart* | Missing destination | Private |
| [`sublime-text-plugins`](https://github.com/ores-truffle-oreslang/sublime-text-plugins) | [`atto-sublime`](https://github.com/atto-lang/atto-sublime) | Candidate (verify history/content) | Public |
| [`oreslang-source.cranelift`](https://github.com/ores-truffle-oreslang/oreslang-source.cranelift) | [`atto-source.cranelift`](https://github.com/atto-lang/atto-source.cranelift) | Candidate (verify history/content) | Private |
| [`oreslang-platform-conformance`](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance) | [`atto-platform-conformance`](https://github.com/atto-lang/atto-platform-conformance) | Candidate (verify history/content) | Private |
| [`.github`](https://github.com/ores-truffle-oreslang/.github) | [`.github`](https://github.com/atto-lang/.github) | Candidate (verify history/content) | Private |
| [`oreslang-otel`](https://github.com/ores-truffle-oreslang/oreslang-otel) | [`atto-otel`](https://github.com/atto-lang/atto-otel) | Candidate (verify history/content) | Private |
| [`oreslang-platform.fpga`](https://github.com/ores-truffle-oreslang/oreslang-platform.fpga) | [`atto-platform.fpga`](https://github.com/atto-lang/atto-platform.fpga) | Candidate (verify history/content) | Private |
| [`oreslang-hot-loading-tests`](https://github.com/ores-truffle-oreslang/oreslang-hot-loading-tests) | [`atto-hot-loading-tests`](https://github.com/atto-lang/atto-hot-loading-tests) | Candidate (verify history/content) | Private |
| [`oreslang-browser`](https://github.com/ores-truffle-oreslang/oreslang-browser) | [`atto-browser`](https://github.com/atto-lang/atto-browser) | Candidate (verify history/content) | Private |
| [`async-testing`](https://github.com/ores-truffle-oreslang/async-testing) | *No existing counterpart* | Missing destination | Private |
| [`oreslang-autograd`](https://github.com/ores-truffle-oreslang/oreslang-autograd) | [`atto-autograd`](https://github.com/atto-lang/atto-autograd) | Candidate (verify history/content) | Private |
| [`oreslang-step-function-demo`](https://github.com/ores-truffle-oreslang/oreslang-step-function-demo) | [`atto-step-function-demo`](https://github.com/atto-lang/atto-step-function-demo) | Candidate (verify history/content) | Private |
| [`oreslang-source.wasm`](https://github.com/ores-truffle-oreslang/oreslang-source.wasm) | [`atto-source.wasm`](https://github.com/atto-lang/atto-source.wasm) | Candidate (verify history/content) | Private |
| [`oreslang-wasm`](https://github.com/ores-truffle-oreslang/oreslang-wasm) | [`atto-wasm`](https://github.com/atto-lang/atto-wasm) | Candidate (verify history/content) | Private |
| [`oreslang-forward-proxy-demo.ores`](https://github.com/ores-truffle-oreslang/oreslang-forward-proxy-demo.ores) | [`atto-forward-proxy-demo.atto`](https://github.com/atto-lang/atto-forward-proxy-demo.atto) | Candidate (verify history/content) | Private |
| [`oreslang-stack-cli`](https://github.com/ores-truffle-oreslang/oreslang-stack-cli) | *No existing counterpart* | Missing destination | Private |
| [`html-js-css-syntax-highlighter`](https://github.com/ores-truffle-oreslang/html-js-css-syntax-highlighter) | *No existing counterpart* | Missing destination | Public |
| [`oreslang-demos`](https://github.com/ores-truffle-oreslang/oreslang-demos) | [`atto-demos`](https://github.com/atto-lang/atto-demos) | Candidate (verify history/content) | Public |

## Explicit mapping decisions required

- **`oreslang-web-browser.ores`**: possible `atto-lang/atto-browser` but semantic equivalence is unverified
- **`oreslang-spin`**: no existing `atto-lang/atto-spin` repository; determine whether to create, consolidate, or retain separately
- **`gh-linguist-fork`**: no existing `atto-lang/gh-linguist-fork` repository; determine whether to create, consolidate, or retain separately
- **`compiler-parity-website`**: no existing `atto-lang/compiler-parity-website` repository; determine whether to create, consolidate, or retain separately
- **`oreslang-durable-objects.ores`**: no existing `atto-lang/atto-durable-objects.atto` repository; determine whether to create, consolidate, or retain separately
- **`oreslang-orie-ieor`**: no existing `atto-lang/atto-orie-ieor` repository; determine whether to create, consolidate, or retain separately
- **`oreslang-hpc-finance`**: no existing `atto-lang/atto-hpc-finance` repository; determine whether to create, consolidate, or retain separately
- **`ores-fm-mac-agent.ores`**: no existing `atto-lang/ores-fm-mac-agent.atto` repository; determine whether to create, consolidate, or retain separately
- **`atto-redis-clone.atto`**: no existing `atto-lang/atto-redis-clone.atto` repository; determine whether to create, consolidate, or retain separately
- **`oreslang-n8n-make-zapier-app`**: possible `atto-lang/atto-automated-jobs` but semantic equivalence is unverified
- **`oreslana.ores`**: no existing `atto-lang/oreslana.atto` repository; determine whether to create, consolidate, or retain separately
- **`async-testing`**: no existing `atto-lang/async-testing` repository; determine whether to create, consolidate, or retain separately
- **`oreslang-stack-cli`**: no existing `atto-lang/atto-stack-cli` repository; determine whether to create, consolidate, or retain separately
- **`html-js-css-syntax-highlighter`**: no existing `atto-lang/html-js-css-syntax-highlighter` repository; determine whether to create, consolidate, or retain separately

**Many-to-one consolidation:** `attolang-language-spec.md` and `oreslang-language-specs.md` both tentatively map to `atto-spec`. Merge their distinct content and histories without treating one as an overwrite of the other. `oreslang-browser` maps to `atto-browser`; `oreslang-web-browser.ores` is *not* automatically merged into it.

## Required preservation for EACH approved pair

1. **Safety / permission check.** Verify authenticated read access to the archived source and write access to the intended Atto destination; check private/public visibility, CODEOWNERS, branch protection, secret handling, licensing, Git LFS and repository rules. Keep archived sources immutable. Do not expose private names or content in public `.github` artifacts.
2. **Capture the real inventory.** Record immutable source/destination default-branch SHAs, ALL Git branches, tags, notes where applicable, GitHub `refs/pull/*/head` and `refs/pull/*/merge` when available, all PR states (open/draft/closed/merged), reviews, review comments, issue comments, labels, assignees, attachments/links, milestones, discussions, and cross-references. Paginate fully. Pin current heads and prove completeness before applying changes.
3. **Back up both ends.** Use authenticated `git fetch` to import Git pack objects, not a file-only snapshot or cherry-pick. Capture original branch and PR-head refs under collision-free Atto migration/archive refs and original tags under namespaced refs. Preserve original **Git object IDs** and parent relationships. Create separate recoverable backups of Atto refs, source refs and any local-only commits; never mirror-push over existing Atto refs.
4. **Integrate source `main` without rewriting history.** On a review branch, perform a real two-parent merge (allow unrelated histories where required); reconcile renamed `.ores` → `.atto` paths and true conflicts semantically. Do not use `-s ours`, squash, rebase original commits, force-update protected branches, or silently remove Atto-only changes. Preserve the original source DAG even if a subsequent Atto-specific adaptation commit is needed.
5. **PR migration is a separate workflow.** Source PR numbers, authorship, original discussion IDs, approvals, and creation timestamps cannot simply be transplanted into Atto's existing PR database. Preserve metadata plus backlinks and exact source PR head SHAs; deduplicate against newer/overlapping Atto PRs; create destination **draft** PRs (or integrate into the canonical Atto PR) for missing work, with fresh tests and fresh approvals. Respect `keep-as-draft` / `not-ready` governance.
6. **Issue migration is a separate workflow.** GitHub native issue transfer is restricted to a single owner; these are different organizations. Archive **all** original issue bodies, comment history, authors, timestamps, state, labels, links and attachments. Recreate unresolved actionable issues in the mapped Atto repo with provenance and link to the immutable source issue; mark duplicates/consolidations explicitly; do not misrepresent recreated comments as authored by original users.
7. **Evidence for every pair.** Verify `git merge-base --is-ancestor <original-source-main-sha> <atto-main-sha>` after the reviewed merge; verify every legacy branch/tag/PR-head SHA is reachable from a documented Atto preservation ref, verify backed-up SHA lists against source inventory, and verify Git LFS payloads where present. Run exact-head positive/negative tests and required native AOT / actor isolation / compiler conformance; record CI and approvals. Partial file equality is not history parity.

## Execution status

- [x] Discover source and destination repository inventories.
- [x] Produce full mapping candidates and exception list.
- [ ] Approve a destination for the 14 exception rows; create missing repositories as appropriate.
- [ ] For every approved pair, fetch and preserve original Git objects and refs using authorized Git transport.
- [ ] Verify branch, tag, PR-head and source-main ancestry after upload.
- [ ] Export the complete historical PR/issue metadata; deduplicate and rehydrate outstanding work in Atto.
- [ ] Perform semantic merges, exact-head test gates and governed approvals.
- [ ] Confirm migrated status from destination `main` and produce signed/pinned receipts.

## Existing migration work and limitations

- [atto-lang/.github issue #11](https://github.com/atto-lang/.github/issues/11) tracks original-history preservation for nine compiler backends.
- [atto-lang/.github draft PR #12](https://github.com/atto-lang/.github/pull/12) supplies a guarded Git migration procedure, **not** completed original-history transfers.
- The linked GitHub connector supports repository reads and individual Git object writes, but not cross-repo raw Git pack transfer. **Do not assert the full migration is done** until an authorized Git client/runner performs actual fetch/push and the receipts above pass.
- Since target repositories already contain new Atto work, **do not transfer/rename the source repository into an occupied destination**. A repository transfer is not a merger of Git history + GitHub collaboration records.

**Invariant:** original data is preserved even if automatic porting of behavior is not yet possible. The final per-repository status must separately report (a) original Git object reachability, (b) branch/tag/PR-head preservation, (c) PR and issue catalog preservation, and (d) semantic integration/test completion.
