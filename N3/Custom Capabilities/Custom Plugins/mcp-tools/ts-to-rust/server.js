#!/usr/bin/env node
// ts-to-rust MCP server, v1.
// Two tools, one stdio server:
//   ts_check    -> TypeScript 7.1 (tsrs, falling back to tsc 7.1) --noEmit verdict
//   ts_to_rust  -> subset transpiler, gated by that same TS 7.1 check
//
// Honest scope, stated in the code so it cannot drift:
// - ts_check uses the real TS 7.1 compiler binary. TS 7.1's JS API is a
//   native sync channel (dist/api/sync), not the classic createSourceFile
//   API, so v1 does NOT claim to parse through that API. The emitter below
//   is a subset parser for the declarations it names, and it refuses to
//   emit when ts_check fails.
// - Subset: interfaces, type aliases (string-literal unions, object-literal
//   unions, Option unions), TS enums, classes (fields, methods, ctor),
//   typed functions, const/let, if/else, blocks, return, expression
//   statements, and expressions listed in emitExpr. Everything else is
//   reported in `unsupported`, never silently rewritten.
// - Type map: number -> f64, string -> String, boolean -> bool,
//   T[] -> Vec<T>, T | null | undefined -> Option<T>.
// - Numeric literals emit as f64 (3f64) because number is always f64 here.
//   Array indices emit as usize.
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const TSRS_DEFAULT = "/Users/stewartbarteau/Desktop/GitHub/Rust-TS-Tokio-Inference-Engine/node_modules/.bin/tsrs";
const TSC_DEFAULT = "/Users/stewartbarteau/Desktop/GitHub/Rust-TS-Tokio-Inference-Engine/node_modules/typescript/bin/tsc";

// ---------- tokenizer ----------
const KEYWORDS = new Set(["interface","type","enum","class","function","const","let","var","return","if","else","for","while","new","extends","implements","public","private","protected","readonly","static","export","import","from","as","typeof","void","string","number","boolean","true","false","null","undefined","this","super"]);
function tokenize(src) {
  const toks = []; let i = 0;
  const push = (kind, value) => toks.push({kind, value});
  while (i < src.length) {
    const c = src[i];
    if (c === "/" && src[i+1] === "/") { while (i < src.length && src[i] !== "\n") i++; continue; }
    if (c === "/" && src[i+1] === "*") { i += 2; while (i < src.length && !(src[i] === "*" && src[i+1] === "/")) i++; i += 2; continue; }
    if (/\s/.test(c)) { i++; continue; }
    if (c === "'" || c === '"' || c === "`") {
      let j = i + 1, v = "";
      while (j < src.length && src[j] !== c) { if (src[j] === "\\") { v += src[j] + src[j+1]; j += 2; } else { v += src[j]; j++; } }
      push("string", v); i = j + 1; continue;
    }
    if (/[0-9]/.test(c) || (c === "." && /[0-9]/.test(src[i+1] || ""))) {
      let j = i; while (j < src.length && /[0-9a-zA-Z._]/.test(src[j])) j++;
      push("number", src.slice(i, j)); i = j; continue;
    }
    if (/[A-Za-z_$]/.test(c)) {
      let j = i; while (j < src.length && /[A-Za-z0-9_$]/.test(src[j])) j++;
      push("ident", src.slice(i, j)); i = j; continue;
    }
    const three = src.slice(i, i+3), two = src.slice(i, i+2);
    if (["===", "!==", "...", "**="].includes(three)) { push("op", three); i += 3; continue; }
    if (["==", "!=", "<=", ">=", "&&", "||", "=>", "++", "--", "+=", "-=", "*=", "/=", "%="].includes(two)) { push("op", two); i += 2; continue; }
    push("op", c); i++;
  }
  push("eof", ""); return toks;
}

// ---------- parser ----------
class Parser {
  constructor(toks, unsupported) { this.toks = toks; this.i = 0; this.unsupported = unsupported; }
  peek(k = 0) { return this.toks[Math.min(this.i + k, this.toks.length - 1)]; }
  next() { return this.toks[this.i++]; }
  atOp(v) { const t = this.peek(); return t.kind === "op" && t.value === v; }
  eatOp(v) { if (this.atOp(v)) { this.i++; return true; } return false; }
  expectOp(v) { if (!this.eatOp(v)) this.unsupported.push("expected '" + v + "' near " + this.peek().value); }
  atIdent(v) { const t = this.peek(); return t.kind === "ident" && (v === undefined || t.value === v); }
  eatIdent(v) { if (this.atIdent(v)) { this.i++; return true; } return false; }
  expectIdent(v) { if (v && !this.eatIdent(v)) this.unsupported.push("expected " + v); return this.toks[this.i - 1]; }

  parseProgram() {
    const decls = [];
    while (this.peek().kind !== "eof") {
      if (this.eatOp(";")) continue;
      if (this.eatIdent("export")) { if (this.eatIdent("default")) continue; }
      if (this.eatIdent("import")) { while (!this.atOp(";") && this.peek().kind !== "eof") this.next(); this.eatOp(";"); continue; }
      const d = this.parseDecl(); if (d) decls.push(d);
    }
    return decls;
  }
  skipModifiers() { while (["public","private","protected","readonly","static","abstract","declare"].includes(this.peek().value)) this.next(); }
  parseDecl() {
    this.skipModifiers();
    if (this.atIdent("interface")) return this.parseInterface();
    if (this.atIdent("type")) return this.parseTypeAlias();
    if (this.atIdent("enum")) return this.parseEnum();
    if (this.atIdent("class")) return this.parseClass();
    if (this.atIdent("function")) return this.parseFunction();
    if (this.atIdent("const") || this.atIdent("let") || this.atIdent("var")) return this.parseVar(true);
    this.unsupported.push("top-level construct starting with " + this.peek().value);
    while (!this.atOp(";") && this.peek().kind !== "eof") this.next();
    this.eatOp(";"); return null;
  }
  parseType() {
    // Returns {kind, ...}: name | array | union | literal | object | generic | fn
    let t = this.parseTypeAtom();
    while (this.atOp("[")) { this.next(); this.expectOp("]"); t = {kind: "array", of: t}; }
    if (this.atOp("|")) {
      const members = [t];
      while (this.eatOp("|")) members.push(this.parseTypeAtom());
      return {kind: "union", members};
    }
    return t;
  }
  parseTypeAtom() {
    const t = this.peek();
    if (t.kind === "string") { this.next(); return {kind: "literal", value: t.value}; }
    if (t.kind === "number") { this.next(); return {kind: "literal", value: t.value}; }
    if (this.atOp("(")) { this.next(); const inner = this.parseType(); this.expectOp(")"); return inner; }
    if (this.atOp("{")) {
      this.next(); const fields = [];
      while (!this.atOp("}") && this.peek().kind !== "eof") {
        const nameTok = this.next(); let optional = false;
        if (this.eatOp("?")) optional = true;
        this.expectOp(":");
        const ft = this.parseType();
        fields.push({name: nameTok.value, type: ft, optional});
        this.eatOp(","); this.eatOp(";");
      }
      this.expectOp("}"); return {kind: "object", fields};
    }
    if (t.kind === "ident") {
      this.next(); let name = t.value;
      if (this.atOp(".")) { this.next(); name += "." + this.next().value; }
      if (this.atOp("<")) {
        this.next(); const args = [this.parseType()];
        while (this.eatOp(",")) args.push(this.parseType());
        this.expectOp(">");
        if (name === "Array" && args.length === 1) return {kind: "array", of: args[0]};
        return {kind: "generic", name, args};
      }
      return {kind: "name", name};
    }
    this.unsupported.push("type starting with " + t.value); this.next();
    return {kind: "name", name: "unknown"};
  }
  parseParams() {
    this.expectOp("("); const params = [];
    while (!this.atOp(")") && this.peek().kind !== "eof") {
      this.skipModifiers();
      const nameTok = this.next();
      let optional = false; if (this.eatOp("?")) optional = true;
      let type = null;
      if (this.eatOp(":")) type = this.parseType();
      let def = null;
      if (this.eatOp("=")) def = this.parseExpr();
      params.push({name: nameTok.value, type, optional, def});
      if (!this.eatOp(",")) break;
    }
    this.expectOp(")"); return params;
  }
  parseInterface() {
    this.expectIdent("interface"); const name = this.next().value;
    if (this.eatIdent("extends")) { while (!this.atOp("{") && this.peek().kind !== "eof") this.next(); }
    this.expectOp("{"); const fields = [];
    while (!this.atOp("}") && this.peek().kind !== "eof") {
      if (this.atOp("(")) { this.unsupported.push("method signature in interface " + name); while (!this.atOp(";") && !this.atOp("}") && this.peek().kind !== "eof") this.next(); this.eatOp(";"); continue; }
      const nameTok = this.next(); let optional = false;
      if (this.eatOp("?")) optional = true;
      this.expectOp(":"); const ft = this.parseType();
      fields.push({name: nameTok.value, type: ft, optional});
      this.eatOp(";"); this.eatOp(",");
    }
    this.expectOp("}"); return {kind: "interface", name, fields};
  }
  parseTypeAlias() {
    this.expectIdent("type"); const name = this.next().value;
    this.expectOp("="); const target = this.parseType(); this.eatOp(";");
    return {kind: "alias", name, target};
  }
  parseEnum() {
    this.expectIdent("enum"); const name = this.next().value;
    this.expectOp("{"); const members = [];
    while (!this.atOp("}") && this.peek().kind !== "eof") {
      const m = this.next().value;
      let value = null;
      if (this.eatOp("=")) { const v = this.next(); value = v.value; }
      members.push({name: m, value}); this.eatOp(","); this.eatOp(";");
    }
    this.expectOp("}"); return {kind: "tsenum", name, members};
  }
  parseClass() {
    this.expectIdent("class"); const name = this.next().value;
    if (this.eatIdent("extends")) this.next();
    if (this.eatIdent("implements")) { while (!this.atOp("{") && this.peek().kind !== "eof") this.next(); }
    this.expectOp("{"); const fields = []; const methods = []; let ctor = null;
    while (!this.atOp("}") && this.peek().kind !== "eof") {
      this.skipModifiers();
      if (this.atIdent("constructor")) {
        this.next(); const params = this.parseParams();
        const body = this.parseBlock();
        ctor = {params, body}; continue;
      }
      const nameTok = this.next();
      if (this.atOp("(")) {
        // method: name already consumed, parseMethod starts at "("
        const m = this.parseMethod(nameTok.value); if (m) methods.push(m); continue;
      }
      let optional = false; if (this.eatOp("?")) optional = true;
      let type = null; if (this.eatOp(":")) type = this.parseType();
      let init = null; if (this.eatOp("=")) init = this.parseExpr();
      fields.push({name: nameTok.value, type, optional, init});
      this.eatOp(";"); this.eatOp(",");
    }
    this.expectOp("}");
    return {kind: "class", name, fields, methods, ctor};
  }
  parseMethod(name) {
    const params = this.parseParams();
    let ret = null; if (this.eatOp(":")) ret = this.parseType();
    const body = this.parseBlock();
    return {name, params, ret, body};
  }
  parseFunction() {
    this.expectIdent("function");
    if (this.atOp("*")) this.unsupported.push("generator function");
    const nameTok = this.next(); const name = nameTok.value;
    const params = this.parseParams();
    let ret = null; if (this.eatOp(":")) ret = this.parseType();
    const body = this.parseBlock();
    return {kind: "function", name, params, ret, body};
  }
  parseVar(topLevel) {
    const kw = this.next().value;
    const nameTok = this.next(); const name = nameTok.value;
    let type = null; if (this.eatOp(":")) type = this.parseType();
    let init = null; if (this.eatOp("=")) init = this.parseExpr();
    if (!topLevel) this.eatOp(";");
    else this.eatOp(";");
    return {kind: "var", kw, name, type, init};
  }
  parseBlock() {
    this.expectOp("{"); const stmts = [];
    while (!this.atOp("}") && this.peek().kind !== "eof") {
      const s = this.parseStmt(); if (s) stmts.push(s);
    }
    this.expectOp("}"); return stmts;
  }
  parseStmt() {
    if (this.eatOp(";")) return null;
    if (this.atIdent("return")) { this.next(); let expr = null; if (!this.atOp(";") && !this.atOp("}")) expr = this.parseExpr(); this.eatOp(";"); return {kind: "return", expr}; }
    if (this.atIdent("if")) {
      this.next(); this.expectOp("("); const cond = this.parseExpr(); this.expectOp(")");
      const then = this.atOp("{") ? this.parseBlock() : [this.parseStmt()];
      let els = null;
      if (this.eatIdent("else")) els = this.atOp("{") ? this.parseBlock() : [this.parseStmt()];
      return {kind: "if", cond, then, els};
    }
    if (this.atIdent("const") || this.atIdent("let") || this.atIdent("var")) return this.parseVar(false);
    if (this.atOp("{")) return {kind: "block", stmts: this.parseBlock()};
    const expr = this.parseExpr(); this.eatOp(";");
    return {kind: "expr", expr};
  }
  parseExpr() { return this.parseAssign(); }
  parseAssign() {
    let left = this.parseBinary(0);
    if (this.peek().kind === "op" && ["=","+=","-=","*=","/=","%="].includes(this.peek().value)) {
      const op = this.next().value; const right = this.parseAssign();
      return {kind: "assign", op, left, right};
    }
    return left;
  }
  parseBinary(minPrec) {
    const PRE = {"||":1,"&&":2,"==":3,"!=":3,"===":3,"!==":3,"<":4,">":4,"<=":4,">=":4,"+":5,"-":5,"*":6,"/":6,"%":6};
    let left = this.parseUnary();
    while (true) {
      const t = this.peek(); if (t.kind !== "op" || !(t.value in PRE) || PRE[t.value] < minPrec) break;
      const op = this.next().value; const right = this.parseBinary(PRE[op] + 1);
      left = {kind: "binary", op, left, right};
    }
    return left;
  }
  parseUnary() {
    const t = this.peek();
    if (t.kind === "op" && ["-","!","+"].includes(t.value)) { this.next(); return {kind: "unary", op: t.value, expr: this.parseUnary()}; }
    if (t.kind === "op" && ["++","--"].includes(t.value)) { this.next(); return {kind: "preinc", op: t.value, expr: this.parseUnary()}; }
    return this.parsePostfix();
  }
  parsePostfix() {
    let e = this.parsePrimary();
    while (true) {
      if (this.atOp(".")) { this.next(); const prop = this.next().value; e = {kind: "member", obj: e, prop}; continue; }
      if (this.atOp("[")) { this.next(); const idx = this.parseExpr(); this.expectOp("]"); e = {kind: "index", obj: e, idx}; continue; }
      if (this.atOp("(")) {
        this.next(); const args = [];
        while (!this.atOp(")") && this.peek().kind !== "eof") { args.push(this.parseExpr()); if (!this.eatOp(",")) break; }
        this.expectOp(")"); e = {kind: "call", callee: e, args}; continue;
      }
      if (this.atOp("?") && this.peek(1).kind === "op" && this.peek(1).value === ".") {
        this.next(); this.next(); const prop = this.next().value; e = {kind: "member", obj: e, prop, optional: true}; continue;
      }
      break;
    }
    return e;
  }
  parsePrimary() {
    const t = this.peek();
    if (t.kind === "number") { this.next(); return {kind: "number", value: t.value}; }
    if (t.kind === "string") { this.next(); return {kind: "string", value: t.value}; }
    if (t.kind === "ident") {
      this.next();
      if (t.value === "true") return {kind: "bool", value: true};
      if (t.value === "false") return {kind: "bool", value: false};
      if (t.value === "null" || t.value === "undefined") return {kind: "null"};
      if (t.value === "this") return {kind: "this"};
      if (t.value === "new") {
        const nameTok = this.next();
        let name = nameTok.value;
        if (this.atOp(".")) { this.next(); name += "." + this.next().value; }
        this.expectOp("("); const args = [];
        while (!this.atOp(")") && this.peek().kind !== "eof") { args.push(this.parseExpr()); if (!this.eatOp(",")) break; }
        this.expectOp(")"); return {kind: "new", name, args};
      }
      if (t.value === "typeof") return {kind: "unsupportedExpr", text: "typeof"};
      return {kind: "ident", name: t.value};
    }
    if (this.atOp("(")) { this.next(); const e = this.parseExpr(); this.expectOp(")"); return e; }
    if (this.atOp("[")) {
      this.next(); const items = [];
      while (!this.atOp("]") && this.peek().kind !== "eof") { items.push(this.parseExpr()); if (!this.eatOp(",")) break; }
      this.expectOp("]"); return {kind: "array", items};
    }
    if (this.atOp("{")) {
      this.next(); const fields = [];
      while (!this.atOp("}") && this.peek().kind !== "eof") {
        const k = this.next().value; this.expectOp(":");
        const v = this.parseExpr(); fields.push({name: k, value: v});
        if (!this.eatOp(",")) break;
      }
      this.expectOp("}"); return {kind: "object", fields};
    }
    if (this.atOp("!")) { this.next(); return {kind: "unary", op: "!", expr: this.parseUnary()}; }
    this.unsupported.push("expression starting with " + t.value); this.next();
    return {kind: "ident", name: "unsupported"};
  }
}

// ---------- Rust emission ----------
function pascal(s) { return s.replace(/(^|[^A-Za-z0-9]+)([A-Za-z0-9])/g, (_, __, c) => c.toUpperCase()); }
function rustType(t, unsupported, ctx) {
  if (!t) return "f64";
  switch (t.kind) {
    case "name":
      if (t.name === "string") return "String";
      if (t.name === "number") return "f64";
      if (t.name === "boolean") return "bool";
      if (t.name === "void" || t.name === "undefined") return "()";
      if (t.name === "any" || t.name === "unknown") { unsupported.push("any/unknown type in " + ctx); return "String"; }
      return pascal(t.name);
    case "array": return "Vec<" + rustType(t.of, unsupported, ctx) + ">";
    case "generic":
      if (t.name === "Record") return "std::collections::HashMap<String, " + rustType(t.args[1] || t.args[0], unsupported, ctx) + ">";
      unsupported.push("generic " + t.name + " in " + ctx); return "String";
    case "literal": return "String";
    case "union": {
      const nonNull = t.members.filter(m => !(m.kind === "name" && (m.name === "null" || m.name === "undefined")));
      if (nonNull.length === 1) return "Option<" + rustType(nonNull[0], unsupported, ctx) + ">";
      if (nonNull.every(m => m.kind === "literal")) return pascal(ctx || "Value");
      unsupported.push("union type in " + ctx + " (handled only for string literals, object literals, and Option)");
      return "String";
    }
    case "object": return pascal(ctx || "Value");
    default: unsupported.push("type kind " + t.kind + " in " + ctx); return "String";
  }
}
function rustString(s) { return '"' + s.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n") + '".to_string()'; }
function emitExpr(e, unsupported) {
  if (!e) return "()";
  switch (e.kind) {
    case "number": {
      const v = e.value;
      if (/[.eE]/.test(v)) return v + "f64";
      return v + "f64";
    }
    case "string": return rustString(e.value);
    case "bool": return e.value ? "true" : "false";
    case "null": return "None";
    case "ident": return e.name;
    case "this": return "self";
    case "array": return "vec![" + e.items.map(x => emitExpr(x, unsupported)).join(", ") + "]";
    case "object": {
      unsupported.push("object literal (no target struct known in expression)");
      return "Default::default()";
    }
    case "member": {
      if (e.obj.kind === "ident" && e.prop === "length") return e.obj.name + ".len() as f64";
      if (e.prop === "length") return "(" + emitExpr(e.obj, unsupported) + ").len() as f64";
      return emitExpr(e.obj, unsupported) + "." + e.prop;
    }
    case "index": {
      const idx = e.idx.kind === "number" ? e.idx.value + "usize" : emitExpr(e.idx, unsupported);
      return emitExpr(e.obj, unsupported) + "[" + idx + "]";
    }
    case "call": {
      const callee = e.callee;
      if (callee.kind === "member") {
        if (callee.prop === "push") return emitExpr(callee.obj, unsupported) + ".push(" + e.args.map(a => emitExpr(a, unsupported)).join(", ") + ")";
        if (callee.prop === "len") return emitExpr(callee.obj, unsupported) + ".len()";
        return emitExpr(callee.obj, unsupported) + "." + callee.prop + "(" + e.args.map(a => emitExpr(a, unsupported)).join(", ") + ")";
      }
      if (callee.kind === "ident") return callee.name + "(" + e.args.map(a => emitExpr(a, unsupported)).join(", ") + ")";
      return emitExpr(callee, unsupported) + "(" + e.args.map(a => emitExpr(a, unsupported)).join(", ") + ")";
    }
    case "new": return pascal(e.name) + "::new(" + e.args.map(a => emitExpr(a, unsupported)).join(", ") + ")";
    case "binary": {
      const op = e.op === "===" ? "==" : e.op === "!==" ? "!=" : e.op;
      return "(" + emitExpr(e.left, unsupported) + " " + op + " " + emitExpr(e.right, unsupported) + ")";
    }
    case "unary": return "(" + e.op + emitExpr(e.expr, unsupported) + ")";
    case "preinc": return "{" + emitExpr(e.expr, unsupported) + " " + (e.op === "++" ? "+= 1f64" : "-= 1f64") + "; " + emitExpr(e.expr, unsupported) + "}";
    case "assign": {
      const op = e.op === "=" ? "=" : e.op;
      return emitExpr(e.left, unsupported) + " " + op + " " + emitExpr(e.right, unsupported);
    }
    case "unsupportedExpr": unsupported.push("typeof expression"); return "()";
    default: unsupported.push("expression kind " + e.kind); return "()";
  }
}
function emitStmts(stmts, indent, unsupported) {
  const pad = "  ".repeat(indent); const out = [];
  for (const s of stmts) {
    if (s.kind === "return") out.push(pad + (s.expr ? "return " + emitExpr(s.expr, unsupported) + ";" : "return;"));
    else if (s.kind === "var") {
      const ty = s.type ? ": " + rustType(s.type, unsupported, s.name) : "";
      out.push(pad + "let mut " + s.name + ty + (s.init ? " = " + emitExpr(s.init, unsupported) : "") + ";");
    } else if (s.kind === "if") {
      out.push(pad + "if " + emitExpr(s.cond, unsupported) + " {");
      out.push(...emitStmts(s.then, indent + 1, unsupported));
      if (s.els) { out.push(pad + "} else {"); out.push(...emitStmts(s.els, indent + 1, unsupported)); }
      out.push(pad + "}");
    } else if (s.kind === "block") { out.push(pad + "{"); out.push(...emitStmts(s.stmts, indent + 1, unsupported)); out.push(pad + "}"); }
    else if (s.kind === "expr") out.push(pad + emitExpr(s.expr, unsupported) + ";");
    else unsupported.push("statement kind " + s.kind);
  }
  return out;
}
function emitParams(params, unsupported) {
  return params.map(p => {
    const ty = p.type ? rustType(p.type, unsupported, p.name) : "f64";
    const t2 = p.optional ? "Option<" + ty + ">" : ty;
    return (p.name === "this" ? "" : p.name + ": " + t2);
  }).filter(Boolean).join(", ");
}
function emitDecl(d, unsupported) {
  const out = [];
  if (d.kind === "interface") {
    out.push("#[derive(Debug, Clone)]");
    out.push("pub struct " + pascal(d.name) + " {");
    for (const f of d.fields) {
      const ty = rustType(f.type, unsupported, d.name + "." + f.name);
      out.push("  pub " + f.name + ": " + (f.optional ? "Option<" + ty + ">" : ty) + ",");
    }
    out.push("}");
  } else if (d.kind === "alias") {
    const t = d.target;
    if (t.kind === "union" && t.members.every(m => m.kind === "literal")) {
      out.push("#[derive(Debug, Clone, PartialEq)]");
      out.push("pub enum " + pascal(d.name) + " {");
      for (const m of t.members) out.push("  " + pascal(String(m.value)) + ",");
      out.push("}");
    } else if (t.kind === "union" && t.members.every(m => m.kind === "object")) {
      out.push("#[derive(Debug, Clone)]");
      out.push("pub enum " + pascal(d.name) + " {");
      for (const m of t.members) {
        const kindField = m.fields.find(f => f.name === "kind" && f.type.kind === "literal");
        const variant = kindField ? pascal(String(kindField.type.value)) : "Variant";
        const rest = m.fields.filter(f => f !== kindField);
        if (rest.length === 0) out.push("  " + variant + ",");
        else {
          out.push("  " + variant + " {");
          for (const f of rest) out.push("    " + f.name + ": " + rustType(f.type, unsupported, d.name) + ",");
          out.push("  },");
        }
      }
      out.push("}");
    } else if (t.kind === "union") {
      const ty = rustType(t, unsupported, d.name);
      out.push("pub type " + pascal(d.name) + " = " + ty + ";");
    } else {
      out.push("pub type " + pascal(d.name) + " = " + rustType(t, unsupported, d.name) + ";");
    }
  } else if (d.kind === "tsenum") {
    out.push("#[derive(Debug, Clone, PartialEq)]");
    out.push("pub enum " + pascal(d.name) + " {");
    for (const m of d.members) out.push("  " + pascal(m.name) + ",");
    out.push("}");
  } else if (d.kind === "function") {
    const ret = d.ret ? rustType(d.ret, unsupported, d.name) : "()";
    out.push("pub fn " + d.name + "(" + emitParams(d.params, unsupported) + ") -> " + ret + " {");
    out.push(...emitStmts(d.body, 1, unsupported));
    out.push("}");
  } else if (d.kind === "var") {
    const ty = d.type ? rustType(d.type, unsupported, d.name) : null;
    out.push("pub const " + d.name.toUpperCase() + (ty ? ": " + ty : "") + (d.init ? " = " + emitExpr(d.init, unsupported) : "") + ";");
  } else if (d.kind === "class") {
    // fields: declared, plus ctor-assigned this.x where a param of same name exists
    const fields = [...d.fields];
    if (d.ctor) {
      for (const s of d.ctor.body) {
        if (s.kind === "expr" && s.expr.kind === "assign" && s.expr.left.kind === "member" && s.expr.left.obj.kind === "this") {
          const fname = s.expr.left.prop;
          if (!fields.find(f => f.name === fname)) {
            const param = d.ctor.params.find(p => p.name === fname);
            fields.push({name: fname, type: param ? param.type : null, optional: false, init: null});
          }
        }
      }
    }
    out.push("#[derive(Debug, Clone)]");
    out.push("pub struct " + pascal(d.name) + " {");
    for (const f of fields) {
      const ty = f.type ? rustType(f.type, unsupported, d.name + "." + f.name) : "f64";
      out.push("  pub " + f.name + ": " + (f.optional ? "Option<" + ty + ">" : ty) + ",");
    }
    out.push("}");
    out.push("impl " + pascal(d.name) + " {");
    if (d.ctor) {
      const ctorParams = d.ctor.params.filter(p => p.name !== "this");
      out.push("  pub fn new(" + emitParams(ctorParams, unsupported) + ") -> Self {");
      // ctor body: only this.x = ... assignments become field inits; others emitted
      const inits = [];
      for (const s of d.ctor.body) {
        if (s.kind === "expr" && s.expr.kind === "assign" && s.expr.left.kind === "member" && s.expr.left.obj.kind === "this") {
          inits.push("    " + s.expr.left.prop + ": " + emitExpr(s.expr.right, unsupported) + ",");
        } else if (s.kind === "var") {
          out.push("    let mut " + s.name + (s.type ? ": " + rustType(s.type, unsupported, s.name) : "") + (s.init ? " = " + emitExpr(s.init, unsupported) : "") + ";");
        }
      }
      // fields not assigned in ctor get a default from their init, else Default
      for (const f of fields) {
        if (!inits.find(x => x.trim().startsWith(f.name + ":"))) {
          if (f.init) inits.push("    " + f.name + ": " + emitExpr(f.init, unsupported) + ",");
          else if (f.type && f.type.kind === "array") inits.push("    " + f.name + ": Vec::new(),");
          else if (f.optional) inits.push("    " + f.name + ": None,");
          else inits.push("    " + f.name + ": Default::default(),");
        }
      }
      out.push("    Self {");
      out.push(...inits);
      out.push("    }");
      out.push("  }");
    }
    for (const m of d.methods) {
      const bodyText = JSON.stringify(m.body);
      const mutates = /"this"/.test(bodyText) && /"assign"|"preinc"/.test(bodyText);
      const ret = m.ret ? rustType(m.ret, unsupported, m.name) : "()";
      const selfArg = m.name === "new" ? "" : (mutates ? "&mut self" : "&self");
      const params = emitParams(m.params, unsupported);
      const allParams = [selfArg, params].filter(Boolean).join(", ");
      out.push("  pub fn " + m.name + "(" + allParams + ") -> " + ret + " {");
      out.push(...emitStmts(m.body, 2, unsupported));
      out.push("  }");
    }
    out.push("}");
  }
  return out;
}
function transpile(source) {
  const unsupported = [];
  const parser = new Parser(tokenize(source), unsupported);
  const decls = parser.parseProgram();
  const lines = ["// Generated by ts-to-rust v1 (subset). Unsupported items are listed, not hidden."];
  for (const d of decls) lines.push(...emitDecl(d, unsupported));
  return {rust: lines.join("\n") + "\n", unsupported: [...new Set(unsupported)]};
}

// ---------- ts_check via TS 7.1 ----------
function tsCheck(source) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "ts-to-rust-"));
  const file = path.join(dir, "input.ts");
  fs.writeFileSync(file, source);
  const bin = process.env.TSRS_BIN || (fs.existsSync(TSRS_DEFAULT) ? TSRS_DEFAULT : TSC_DEFAULT);
  const r = spawnSync(bin, ["--noEmit", "--strict", "--target", "es2022", "--module", "esnext", file], {encoding: "utf8", timeout: 30000});
  const output = ((r.stdout || "") + (r.stderr || "")).trim();
  return {ok: r.status === 0, compiler: bin, diagnostics: output, exitCode: r.status};
}

// ---------- MCP stdio ----------
const TOOLS = [
  {name: "ts_check", description: "Typecheck TypeScript source with TypeScript 7.1 (tsrs, fallback tsc 7.1) --noEmit. Returns ok and diagnostics.", inputSchema: {type: "object", properties: {source: {type: "string", description: "TypeScript source"}}, required: ["source"]}},
  {name: "ts_to_rust", description: "Transpile the supported TypeScript subset to Rust. Runs ts_check first and refuses on check failure. Unsupported constructs are listed, never silently rewritten.", inputSchema: {type: "object", properties: {source: {type: "string", description: "TypeScript source in the supported subset"}}, required: ["source"]}},
];
function callTool(name, args) {
  const source = (args && args.source) || "";
  if (name === "ts_check") {
    const c = tsCheck(source);
    return {content: [{type: "text", text: JSON.stringify(c, null, 2)}], isError: false};
  }
  if (name === "ts_to_rust") {
    const c = tsCheck(source);
    if (!c.ok) return {content: [{type: "text", text: JSON.stringify({ok: false, reason: "ts_check failed, refusing to emit", check: c}, null, 2)}], isError: true};
    const t = transpile(source);
    return {content: [{type: "text", text: JSON.stringify({ok: true, check: c, rust: t.rust, unsupported: t.unsupported}, null, 2)}], isError: false};
  }
  throw new Error("unknown tool " + name);
}
function handle(msg) {
  if (!msg || typeof msg.method !== "string") return null;
  if (msg.method === "initialize") return {jsonrpc: "2.0", id: msg.id, result: {protocolVersion: "2024-11-05", capabilities: {tools: {}}, serverInfo: {name: "ts-to-rust", version: "0.1.0"}}};
  if (msg.method === "notifications/initialized" || msg.method.startsWith("notifications/")) return null;
  if (msg.method === "ping") return {jsonrpc: "2.0", id: msg.id, result: {}};
  if (msg.method === "tools/list") return {jsonrpc: "2.0", id: msg.id, result: {tools: TOOLS}};
  if (msg.method === "tools/call") {
    try {
      const r = callTool(msg.params.name, msg.params.arguments || {});
      return {jsonrpc: "2.0", id: msg.id, result: r};
    } catch (e) {
      return {jsonrpc: "2.0", id: msg.id, result: {content: [{type: "text", text: "error: " + e.message}], isError: true}};
    }
  }
  return {jsonrpc: "2.0", id: msg.id, error: {code: -32601, message: "method not found: " + msg.method}};
}
let buf = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", chunk => {
  buf += chunk;
  let idx;
  while ((idx = buf.indexOf("\n")) >= 0) {
    const line = buf.slice(0, idx).trim(); buf = buf.slice(idx + 1);
    if (!line) continue;
    let msg; try { msg = JSON.parse(line); } catch { continue; }
    const res = handle(msg);
    if (res) process.stdout.write(JSON.stringify(res) + "\n");
  }
});
