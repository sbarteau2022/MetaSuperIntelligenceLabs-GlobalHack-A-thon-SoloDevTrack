# Universal forward pass: open gates

1. Bind model seat identity to verified shard/index/config/tokenizer bytes. The current `seat_identity` omits weight shards. Rebuild the server, attest its binary SHA, and prove a one-bit shard mutation changes the seat while unchanged bytes reproduce it. Do not reuse old tensor K/V across revisions.
2. Reset local model attention state at each independent request, or require an explicit authenticated continuation identity. Run two identical requests in one process and compare both with a fresh-process result; force an intermediate failing request and repeat.
3. Replace JSON `/v1/tokenize` and `/v1/forward` with a bounded typed binary model I/O contract. Reject malformed lengths, unknown versions, partial frames, and receipt drift. The existing AMST response alone does not make the request boundary binary.
4. Choose and implement exact Glimmer tensor K/V export/inject with the full identity, or explicitly exclude Glimmer from nested K/V claims. The user-defined system requires nested tensor K/V; exclusion cannot certify that system. Test uninterrupted versus persisted/restarted continuation on real weights.
5. Compile and execute a validated operator graph. The current universal decoder is admission scaffolding only. Prove a changed compatible checkpoint runs without source edits and a second architecture uses the same binary ABI.
6. Run the independent full-model logit oracle, then one harness→automaton→active `globalmind-mcp`→Surreal recall→model forward with a no-recall control, answer judge, and trajectory receipt. Record every failed gate; no task-quality or speed claim before this.

No GPU sweep or long autonomous run was launched. The already-running Glimmer server was left untouched.
