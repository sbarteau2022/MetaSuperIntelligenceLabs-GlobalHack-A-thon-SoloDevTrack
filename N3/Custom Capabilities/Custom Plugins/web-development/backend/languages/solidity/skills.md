# solidity: assume an adversary, spend gas carefully
**Done means these pass:** `forge fmt --check`, `forge build` with no warnings, `forge test` (including fuzz and invariant tests), `slither .` with no unreviewed high/medium findings, and `solhint`.
## Rules
1. Pin the compiler (`pragma solidity 0.8.x;` exact or narrow) and use 0.8+ for checked arithmetic; use `unchecked` only in a proven, commented hot spot. Use custom errors (`error Unauthorized();`) instead of long revert strings.
2. Checks-effects-interactions: validate, update state, then make external calls; add `nonReentrant` (OpenZeppelin `ReentrancyGuard`) to functions that send value or call untrusted code; prefer pull payments (users withdraw) over pushing ether to arbitrary addresses.
3. Access control on every state-changing function (`Ownable2Step`/`AccessControl`); never `tx.origin` for authorisation; initialise upgradeable contracts exactly once (`initializer`) and lock implementations (`_disableInitializers()`); follow storage-layout rules for upgrades (append-only, use gaps/namespaced storage).
4. Check the result of low-level calls (`(bool ok, ) = to.call{value: v}(""); require(ok);`); use `SafeERC20` for tokens (non-standard returns, fee-on-transfer, rebasing); never assume `balanceOf` equals what you transferred; avoid `transfer`/`send` gas assumptions.
5. Validate inputs (zero address, array lengths, bounds); no unbounded loops over user-controlled arrays (denial of service); avoid relying on `block.timestamp` for tight randomness/deadlines, and never use on-chain data as secure randomness; use oracles with staleness and sanity checks (Chainlink `updatedAt`, `answer > 0`).
6. Events for every state change that off-chain systems need; indexed fields where useful; do not emit secrets (everything on-chain is public).
7. Gas: `calldata` for external array/struct params, `immutable`/`constant` for fixed values, pack storage variables, cache storage reads in memory, avoid redundant SLOADs; measure with `forge snapshot` before and after.
8. Signatures: use EIP-712 typed data with domain separation, nonces and deadlines; use `ECDSA.recover` from OpenZeppelin (rejects malleable signatures); guard against replay across chains/contracts.
9. Test properties, not just examples: fuzz inputs, invariants (total supply, solvency), fork tests for integrations; test failure paths (`vm.expectRevert`) and boundary values; get an independent audit before holding real funds.
10. Use audited libraries (OpenZeppelin) instead of re-implementing standards; keep contracts small and composable.
## Pitfalls
Reentrancy through ERC777/ERC721 hooks and callbacks; precision loss from division before multiplication; unchecked return values; storage collisions in proxies; front-running and sandwiching of price-dependent calls; `delegatecall` to untrusted code.
## Versions
Pinned: Solidity 0.8.37 (0.8.37 (Sep 2026); checked 2026-10-07, source: web, Sep 2026). This skill assumes at least Solidity 0.8.20.
Rule: first run `solc --version` and read the project's pin (pragma solidity, foundry.toml solc_version). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the Solidity documentation (security considerations), ConsenSys Smart Contract Best Practices, the SWC registry, OpenZeppelin docs.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/solidity.md).

Canonical source: `plugins/web-development/backend/languages/solidity/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/solidity.md` (reference; do not author).
