var __defProp = Object.defineProperty;
var __name = (target, value) => __defProp(target, "name", { value, configurable: true });

// build/worker/shim.mjs
import me from "./f43cc6f9fd6bef2b4451a213344553bbb06ff61f-index.wasm";
import { WorkerEntrypoint as xe } from "cloudflare:workers";
var H = Object.defineProperty;
var N = /* @__PURE__ */ __name((e, t) => {
  for (var n in t) H(e, n, { get: t[n], enumerable: true });
}, "N");
var y = {};
N(y, { ContainerStartupOptions: /* @__PURE__ */ __name(() => S, "ContainerStartupOptions"), MinifyConfig: /* @__PURE__ */ __name(() => k, "MinifyConfig"), R2Range: /* @__PURE__ */ __name(() => R, "R2Range"), __wbg_String_8564e559799eccda: /* @__PURE__ */ __name(() => G, "__wbg_String_8564e559799eccda"), __wbg___wbindgen_debug_string_4687d8d8c2017d52: /* @__PURE__ */ __name(() => Y, "__wbg___wbindgen_debug_string_4687d8d8c2017d52"), __wbg___wbindgen_is_function_1f9d30630b8b1d3d: /* @__PURE__ */ __name(() => K, "__wbg___wbindgen_is_function_1f9d30630b8b1d3d"), __wbg___wbindgen_is_null_or_undefined_f9435d56bdcc06df: /* @__PURE__ */ __name(() => Q, "__wbg___wbindgen_is_null_or_undefined_f9435d56bdcc06df"), __wbg___wbindgen_is_undefined_8865fb403f8fe9d8: /* @__PURE__ */ __name(() => Z, "__wbg___wbindgen_is_undefined_8865fb403f8fe9d8"), __wbg___wbindgen_reinit_3f79fdb34dee3ea9: /* @__PURE__ */ __name(() => tt, "__wbg___wbindgen_reinit_3f79fdb34dee3ea9"), __wbg___wbindgen_string_get_0380ccaa2f57f0d9: /* @__PURE__ */ __name(() => et, "__wbg___wbindgen_string_get_0380ccaa2f57f0d9"), __wbg___wbindgen_throw_41e9ee4f547fc59a: /* @__PURE__ */ __name(() => nt, "__wbg___wbindgen_throw_41e9ee4f547fc59a"), __wbg__wbg_cb_unref_dcc1a90847f04c41: /* @__PURE__ */ __name(() => rt, "__wbg__wbg_cb_unref_dcc1a90847f04c41"), __wbg_buffer_56ec2905a66f58b9: /* @__PURE__ */ __name(() => _t, "__wbg_buffer_56ec2905a66f58b9"), __wbg_byobRequest_54f7ce41d584549a: /* @__PURE__ */ __name(() => it, "__wbg_byobRequest_54f7ce41d584549a"), __wbg_byteLength_9f985b8373f1f3fe: /* @__PURE__ */ __name(() => ot, "__wbg_byteLength_9f985b8373f1f3fe"), __wbg_byteOffset_0bd5a6a03553639c: /* @__PURE__ */ __name(() => st, "__wbg_byteOffset_0bd5a6a03553639c"), __wbg_call_187d372bd5fdd4aa: /* @__PURE__ */ __name(() => ct, "__wbg_call_187d372bd5fdd4aa"), __wbg_cause_19276d032a32d4bd: /* @__PURE__ */ __name(() => ut, "__wbg_cause_19276d032a32d4bd"), __wbg_cf_b47fe5ff6d015a0d: /* @__PURE__ */ __name(() => ft, "__wbg_cf_b47fe5ff6d015a0d"), __wbg_close_185286988f30f252: /* @__PURE__ */ __name(() => at, "__wbg_close_185286988f30f252"), __wbg_close_748f26698dabe269: /* @__PURE__ */ __name(() => bt, "__wbg_close_748f26698dabe269"), __wbg_enqueue_4a01c128f9a0de16: /* @__PURE__ */ __name(() => gt, "__wbg_enqueue_4a01c128f9a0de16"), __wbg_error_a30e98d44009f1bb: /* @__PURE__ */ __name(() => dt, "__wbg_error_a30e98d44009f1bb"), __wbg_error_c9cf3fc2064683a9: /* @__PURE__ */ __name(() => wt, "__wbg_error_c9cf3fc2064683a9"), __wbg_get_88983e33467eae60: /* @__PURE__ */ __name(() => lt, "__wbg_get_88983e33467eae60"), __wbg_headers_6930a8bd09b630de: /* @__PURE__ */ __name(() => pt, "__wbg_headers_6930a8bd09b630de"), __wbg_instanceId_9b67ff33ae8bef8a: /* @__PURE__ */ __name(() => ht, "__wbg_instanceId_9b67ff33ae8bef8a"), __wbg_instanceof_Error_80a725f81f2e102d: /* @__PURE__ */ __name(() => yt, "__wbg_instanceof_Error_80a725f81f2e102d"), __wbg_length_7f3c00c40364105e: /* @__PURE__ */ __name(() => mt, "__wbg_length_7f3c00c40364105e"), __wbg_message_5f8387f0c32b90a7: /* @__PURE__ */ __name(() => xt, "__wbg_message_5f8387f0c32b90a7"), __wbg_method_6d02ee0808a7de31: /* @__PURE__ */ __name(() => It, "__wbg_method_6d02ee0808a7de31"), __wbg_name_e2eac7cdfa054f65: /* @__PURE__ */ __name(() => vt, "__wbg_name_e2eac7cdfa054f65"), __wbg_new_343a093a3c2ffb4e: /* @__PURE__ */ __name(() => Et, "__wbg_new_343a093a3c2ffb4e"), __wbg_new_617a8cdb8bb1130e: /* @__PURE__ */ __name(() => jt, "__wbg_new_617a8cdb8bb1130e"), __wbg_new_83ca405eae412a62: /* @__PURE__ */ __name(() => Ft, "__wbg_new_83ca405eae412a62"), __wbg_new_typed_b01cb72a8af741a3: /* @__PURE__ */ __name(() => Wt, "__wbg_new_typed_b01cb72a8af741a3"), __wbg_new_with_byte_offset_and_length_2f5d7fc2a828b74d: /* @__PURE__ */ __name(() => St, "__wbg_new_with_byte_offset_and_length_2f5d7fc2a828b74d"), __wbg_new_with_length_3da0ad195f6f63ba: /* @__PURE__ */ __name(() => kt, "__wbg_new_with_length_3da0ad195f6f63ba"), __wbg_new_with_opt_buffer_source_and_init_c1b859a23c80afca: /* @__PURE__ */ __name(() => Rt, "__wbg_new_with_opt_buffer_source_and_init_c1b859a23c80afca"), __wbg_new_with_opt_readable_stream_and_init_df114f2d364f216d: /* @__PURE__ */ __name(() => At, "__wbg_new_with_opt_readable_stream_and_init_df114f2d364f216d"), __wbg_new_with_opt_str_and_init_33bef1b272950372: /* @__PURE__ */ __name(() => zt, "__wbg_new_with_opt_str_and_init_33bef1b272950372"), __wbg_queueMicrotask_9833f9a49df95a49: /* @__PURE__ */ __name(() => Ot, "__wbg_queueMicrotask_9833f9a49df95a49"), __wbg_queueMicrotask_a72f977e97f23c5f: /* @__PURE__ */ __name(() => Tt, "__wbg_queueMicrotask_a72f977e97f23c5f"), __wbg_resolve_0076e10020304ede: /* @__PURE__ */ __name(() => Mt, "__wbg_resolve_0076e10020304ede"), __wbg_respond_c102fabcc79e5ef0: /* @__PURE__ */ __name(() => Ut, "__wbg_respond_c102fabcc79e5ef0"), __wbg_set_145a351398b48c65: /* @__PURE__ */ __name(() => qt, "__wbg_set_145a351398b48c65"), __wbg_set_575d3ddb70fe831d: /* @__PURE__ */ __name(() => Dt, "__wbg_set_575d3ddb70fe831d"), __wbg_set_af5932e3f78b5648: /* @__PURE__ */ __name(() => Lt, "__wbg_set_af5932e3f78b5648"), __wbg_set_criticalError_3904d60fe2a0b05b: /* @__PURE__ */ __name(() => Ct, "__wbg_set_criticalError_3904d60fe2a0b05b"), __wbg_set_headers_69ef7c46a2e6c742: /* @__PURE__ */ __name(() => Bt, "__wbg_set_headers_69ef7c46a2e6c742"), __wbg_set_instanceId_9ad5caac0a4e519c: /* @__PURE__ */ __name(() => $t, "__wbg_set_instanceId_9ad5caac0a4e519c"), __wbg_set_status_678f122cbe435d95: /* @__PURE__ */ __name(() => Nt, "__wbg_set_status_678f122cbe435d95"), __wbg_set_wasm: /* @__PURE__ */ __name(() => $, "__wbg_set_wasm"), __wbg_static_accessor_GLOBAL_266715b9d96ba635: /* @__PURE__ */ __name(() => Vt, "__wbg_static_accessor_GLOBAL_266715b9d96ba635"), __wbg_static_accessor_GLOBAL_THIS_10fb7dc1ae063179: /* @__PURE__ */ __name(() => Pt, "__wbg_static_accessor_GLOBAL_THIS_10fb7dc1ae063179"), __wbg_static_accessor_INIT_STATE_e3fce2ad8a08e94e: /* @__PURE__ */ __name(() => Ht, "__wbg_static_accessor_INIT_STATE_e3fce2ad8a08e94e"), __wbg_static_accessor_SELF_0b583911f537483a: /* @__PURE__ */ __name(() => Jt, "__wbg_static_accessor_SELF_0b583911f537483a"), __wbg_static_accessor_WINDOW_d7f903d1508cbdc4: /* @__PURE__ */ __name(() => Xt, "__wbg_static_accessor_WINDOW_d7f903d1508cbdc4"), __wbg_text_aaf360cc4f1c0167: /* @__PURE__ */ __name(() => Gt, "__wbg_text_aaf360cc4f1c0167"), __wbg_then_c949d5a25a4e78f8: /* @__PURE__ */ __name(() => Yt, "__wbg_then_c949d5a25a4e78f8"), __wbg_then_e71170d78fcf8954: /* @__PURE__ */ __name(() => Kt, "__wbg_then_e71170d78fcf8954"), __wbg_url_7bfada5c2297bc70: /* @__PURE__ */ __name(() => Qt, "__wbg_url_7bfada5c2297bc70"), __wbg_view_9c570f33e8d6ab96: /* @__PURE__ */ __name(() => Zt, "__wbg_view_9c570f33e8d6ab96"), __wbindgen_generic_0000000000000001: /* @__PURE__ */ __name(() => te, "__wbindgen_generic_0000000000000001"), __wbindgen_generic_0000000000000002: /* @__PURE__ */ __name(() => ee, "__wbindgen_generic_0000000000000002"), __wbindgen_object_clone_ref: /* @__PURE__ */ __name(() => ne, "__wbindgen_object_clone_ref"), __wbindgen_object_drop_ref: /* @__PURE__ */ __name(() => re, "__wbindgen_object_drop_ref"), __worker_init_state: /* @__PURE__ */ __name(() => X, "__worker_init_state"), fetch: /* @__PURE__ */ __name(() => C, "fetch") });
var D = {};
N(D, { state: /* @__PURE__ */ __name(() => q, "state") });
var q = globalThis.__worker_init_state = { criticalError: false, instanceId: 0 };
var S = class {
  static {
    __name(this, "S");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, se.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    c(), _.__wbg_containerstartupoptions_free(t, 0);
  }
  get enableInternet() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let t;
    return c(), t = _.__wbg_get_containerstartupoptions_enableInternet(this.__wbg_ptr), t === 16777215 ? void 0 : t !== 0;
  }
  get entrypoint() {
    try {
      if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
      let o = _.__wbindgen_add_to_stack_pointer(-16);
      c(), _.__wbg_get_containerstartupoptions_entrypoint(o, this.__wbg_ptr);
      var t = f().getInt32(o + 0, true), n = f().getInt32(o + 4, true), r = de(t, n);
      return _.__wbindgen_export5(t, n * 4, 4), r;
    } finally {
      _.__wbindgen_add_to_stack_pointer(16);
    }
  }
  get env() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let t;
    return c(), t = _.__wbg_get_containerstartupoptions_env(this.__wbg_ptr), p(t);
  }
  set enableInternet(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    c(), _.__wbg_set_containerstartupoptions_enableInternet(this.__wbg_ptr, g(t) ? 16777215 : t ? 1 : 0);
  }
  set entrypoint(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let n = pe(t, _.__wbindgen_export), r = h;
    c(), _.__wbg_set_containerstartupoptions_entrypoint(this.__wbg_ptr, n, r);
  }
  set env(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    c(), _.__wbg_set_containerstartupoptions_env(this.__wbg_ptr, s(t));
  }
};
Symbol.dispose && (S.prototype[Symbol.dispose] = S.prototype.free);
var O = class {
  static {
    __name(this, "O");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, ce.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    c(), _.__wbg_intounderlyingbytesource_free(t, 0);
  }
  get autoAllocateChunkSize() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let t;
    return c(), t = _.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr), t >>> 0;
  }
  cancel() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let t = this.__destroy_into_raw();
    c(), _.intounderlyingbytesource_cancel(t);
  }
  pull(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let n;
    return c(), n = _.intounderlyingbytesource_pull(this.__wbg_ptr, s(t)), p(n);
  }
  start(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    c(), _.intounderlyingbytesource_start(this.__wbg_ptr, s(t));
  }
  get type() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let t;
    return c(), t = _.intounderlyingbytesource_type(this.__wbg_ptr), oe[t];
  }
};
Symbol.dispose && (O.prototype[Symbol.dispose] = O.prototype.free);
var T = class {
  static {
    __name(this, "T");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, ue.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    c(), _.__wbg_intounderlyingsink_free(t, 0);
  }
  abort(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let n = this.__destroy_into_raw(), r;
    return c(), r = _.intounderlyingsink_abort(n, s(t)), p(r);
  }
  close() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let t = this.__destroy_into_raw(), n;
    return c(), n = _.intounderlyingsink_close(t), p(n);
  }
  write(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let n;
    return c(), n = _.intounderlyingsink_write(this.__wbg_ptr, s(t)), p(n);
  }
};
Symbol.dispose && (T.prototype[Symbol.dispose] = T.prototype.free);
var M = class {
  static {
    __name(this, "M");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, fe.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    c(), _.__wbg_intounderlyingsource_free(t, 0);
  }
  cancel() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let t = this.__destroy_into_raw();
    c(), _.intounderlyingsource_cancel(t);
  }
  pull(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let n;
    return c(), n = _.intounderlyingsource_pull(this.__wbg_ptr, s(t)), p(n);
  }
};
Symbol.dispose && (M.prototype[Symbol.dispose] = M.prototype.free);
var k = class {
  static {
    __name(this, "k");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, ae.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    c(), _.__wbg_minifyconfig_free(t, 0);
  }
  get css() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let t;
    return c(), t = _.__wbg_get_minifyconfig_css(this.__wbg_ptr), t !== 0;
  }
  get html() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let t;
    return c(), t = _.__wbg_get_minifyconfig_html(this.__wbg_ptr), t !== 0;
  }
  get js() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    let t;
    return c(), t = _.__wbg_get_minifyconfig_js(this.__wbg_ptr), t !== 0;
  }
  set css(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    c(), _.__wbg_set_minifyconfig_css(this.__wbg_ptr, t);
  }
  set html(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    c(), _.__wbg_set_minifyconfig_html(this.__wbg_ptr, t);
  }
  set js(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    c(), _.__wbg_set_minifyconfig_js(this.__wbg_ptr, t);
  }
};
Symbol.dispose && (k.prototype[Symbol.dispose] = k.prototype.free);
var R = class {
  static {
    __name(this, "R");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, be.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    c(), _.__wbg_r2range_free(t, 0);
  }
  get length() {
    try {
      if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
      let r = _.__wbindgen_add_to_stack_pointer(-16);
      c(), _.__wbg_get_r2range_length(r, this.__wbg_ptr);
      var t = f().getInt32(r + 0, true), n = f().getFloat64(r + 8, true);
      return t === 0 ? void 0 : n;
    } finally {
      _.__wbindgen_add_to_stack_pointer(16);
    }
  }
  get offset() {
    try {
      if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
      let r = _.__wbindgen_add_to_stack_pointer(-16);
      c(), _.__wbg_get_r2range_offset(r, this.__wbg_ptr);
      var t = f().getInt32(r + 0, true), n = f().getFloat64(r + 8, true);
      return t === 0 ? void 0 : n;
    } finally {
      _.__wbindgen_add_to_stack_pointer(16);
    }
  }
  get suffix() {
    try {
      if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
      let r = _.__wbindgen_add_to_stack_pointer(-16);
      c(), _.__wbg_get_r2range_suffix(r, this.__wbg_ptr);
      var t = f().getInt32(r + 0, true), n = f().getFloat64(r + 8, true);
      return t === 0 ? void 0 : n;
    } finally {
      _.__wbindgen_add_to_stack_pointer(16);
    }
  }
  set length(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    c(), _.__wbg_set_r2range_length(this.__wbg_ptr, !g(t), g(t) ? 0 : t);
  }
  set offset(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    c(), _.__wbg_set_r2range_offset(this.__wbg_ptr, !g(t), g(t) ? 0 : t);
  }
  set suffix(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== u) throw new Error("Invalid stale object from previous Wasm instance");
    c(), _.__wbg_set_r2range_suffix(this.__wbg_ptr, !g(t), g(t) ? 0 : t);
  }
};
Symbol.dispose && (R.prototype[Symbol.dispose] = R.prototype.free);
function J() {
  u++, m = null, E = null, typeof F < "u" && (F = 0), typeof h < "u" && (h = 0), typeof d < "u" && (d = new Array(1024).fill(void 0), d = d.concat([void 0, null, true, false]), typeof x < "u" && (x = d.length)), B = false, wasmInstance = new WebAssembly.Instance(wasmModule, __wbg_get_imports()), _ = wasmInstance.exports, _.__wbindgen_start();
}
__name(J, "J");
function X() {
  let e;
  return c(), e = _.__worker_init_state(), p(e);
}
__name(X, "X");
function C(e, t, n) {
  let r;
  return c(), r = _.fetch(s(e), s(t), s(n)), p(r);
}
__name(C, "C");
function G(e, t) {
  let n = String(i(t)), r = A(n, _.__wbindgen_export, _.__wbindgen_export2), o = h;
  f().setInt32(e + 4, o, true), f().setInt32(e + 0, r, true);
}
__name(G, "G");
function Y(e, t) {
  let n = L(i(t)), r = A(n, _.__wbindgen_export, _.__wbindgen_export2), o = h;
  f().setInt32(e + 4, o, true), f().setInt32(e + 0, r, true);
}
__name(Y, "Y");
function K(e) {
  return typeof i(e) == "function";
}
__name(K, "K");
function Q(e) {
  return i(e) == null;
}
__name(Q, "Q");
function Z(e) {
  return i(e) === void 0;
}
__name(Z, "Z");
function tt() {
  B = true;
}
__name(tt, "tt");
function et(e, t) {
  let n = i(t), r = typeof n == "string" ? n : void 0;
  var o = g(r) ? 0 : A(r, _.__wbindgen_export, _.__wbindgen_export2), b = h;
  f().setInt32(e + 4, b, true), f().setInt32(e + 0, o, true);
}
__name(et, "et");
function nt(e, t) {
  throw new Error(I(e, t));
}
__name(nt, "nt");
function rt(e) {
  i(e)._wbg_cb_unref();
}
__name(rt, "rt");
function _t(e) {
  let t = i(e).buffer;
  return s(t);
}
__name(_t, "_t");
function it(e) {
  let t = i(e).byobRequest;
  return g(t) ? 0 : s(t);
}
__name(it, "it");
function ot(e) {
  return i(e).byteLength;
}
__name(ot, "ot");
function st(e) {
  return i(e).byteOffset;
}
__name(st, "st");
function ct() {
  return w(function(e, t, n) {
    let r = i(e).call(i(t), i(n));
    return s(r);
  }, arguments);
}
__name(ct, "ct");
function ut(e) {
  let t = i(e).cause;
  return s(t);
}
__name(ut, "ut");
function ft() {
  return w(function(e) {
    let t = i(e).cf;
    return g(t) ? 0 : s(t);
  }, arguments);
}
__name(ft, "ft");
function at() {
  return w(function(e) {
    i(e).close();
  }, arguments);
}
__name(at, "at");
function bt() {
  return w(function(e) {
    i(e).close();
  }, arguments);
}
__name(bt, "bt");
function gt() {
  return w(function(e, t) {
    i(e).enqueue(i(t));
  }, arguments);
}
__name(gt, "gt");
function dt(e, t) {
  console.error(i(e), i(t));
}
__name(dt, "dt");
function wt(e) {
  console.error(i(e));
}
__name(wt, "wt");
function lt() {
  return w(function(e, t) {
    let n = Reflect.get(i(e), i(t));
    return g(n) ? 0 : s(n);
  }, arguments);
}
__name(lt, "lt");
function pt(e) {
  let t = i(e).headers;
  return s(t);
}
__name(pt, "pt");
function ht(e) {
  return i(e).instanceId;
}
__name(ht, "ht");
function yt(e) {
  let t;
  try {
    t = i(e) instanceof Error;
  } catch {
    t = false;
  }
  return t;
}
__name(yt, "yt");
function mt(e) {
  return i(e).length;
}
__name(mt, "mt");
function xt(e) {
  let t = i(e).message;
  return s(t);
}
__name(xt, "xt");
function It(e, t) {
  let n = i(t).method, r = A(n, _.__wbindgen_export, _.__wbindgen_export2), o = h;
  f().setInt32(e + 4, o, true), f().setInt32(e + 0, r, true);
}
__name(It, "It");
function vt(e) {
  let t = i(e).name;
  return s(t);
}
__name(vt, "vt");
function Et(e, t) {
  let n = new Error(I(e, t));
  return s(n);
}
__name(Et, "Et");
function jt() {
  let e = new Object();
  return s(e);
}
__name(jt, "jt");
function Ft() {
  return w(function() {
    let e = new Headers();
    return s(e);
  }, arguments);
}
__name(Ft, "Ft");
function Wt(e, t) {
  try {
    var n = { a: e, b: t }, r = /* @__PURE__ */ __name((b, a) => {
      let l = n.a;
      n.a = 0;
      try {
        return _e(l, n.b, b, a);
      } finally {
        n.a = l;
      }
    }, "r");
    let o = new Promise(r);
    return s(o);
  } finally {
    n.a = 0;
  }
}
__name(Wt, "Wt");
function St(e, t, n) {
  let r = new Uint8Array(i(e), t >>> 0, n >>> 0);
  return s(r);
}
__name(St, "St");
function kt(e) {
  let t = new Uint8Array(e >>> 0);
  return s(t);
}
__name(kt, "kt");
function Rt() {
  return w(function(e, t) {
    let n = new Response(i(e), i(t));
    return s(n);
  }, arguments);
}
__name(Rt, "Rt");
function At() {
  return w(function(e, t) {
    let n = new Response(i(e), i(t));
    return s(n);
  }, arguments);
}
__name(At, "At");
function zt() {
  return w(function(e, t, n) {
    let r = new Response(e === 0 ? void 0 : I(e, t), i(n));
    return s(r);
  }, arguments);
}
__name(zt, "zt");
function Ot(e) {
  let t = i(e).queueMicrotask;
  return s(t);
}
__name(Ot, "Ot");
function Tt(e) {
  queueMicrotask(i(e));
}
__name(Tt, "Tt");
function Mt(e) {
  let t = Promise.resolve(i(e));
  return s(t);
}
__name(Mt, "Mt");
function Ut() {
  return w(function(e, t) {
    i(e).respond(t >>> 0);
  }, arguments);
}
__name(Ut, "Ut");
function qt() {
  return w(function(e, t, n) {
    return Reflect.set(i(e), i(t), i(n));
  }, arguments);
}
__name(qt, "qt");
function Dt(e, t, n) {
  i(e).set(we(t, n));
}
__name(Dt, "Dt");
function Lt() {
  return w(function(e, t, n, r, o) {
    i(e).set(I(t, n), I(r, o));
  }, arguments);
}
__name(Lt, "Lt");
function Ct(e, t) {
  i(e).criticalError = t !== 0;
}
__name(Ct, "Ct");
function Bt(e, t) {
  i(e).headers = i(t);
}
__name(Bt, "Bt");
function $t(e, t) {
  i(e).instanceId = t >>> 0;
}
__name($t, "$t");
function Nt(e, t) {
  i(e).status = t;
}
__name(Nt, "Nt");
function Vt() {
  let e = typeof global > "u" ? null : global;
  return g(e) ? 0 : s(e);
}
__name(Vt, "Vt");
function Pt() {
  let e = typeof globalThis > "u" ? null : globalThis;
  return g(e) ? 0 : s(e);
}
__name(Pt, "Pt");
function Ht() {
  return s(q);
}
__name(Ht, "Ht");
function Jt() {
  let e = typeof self > "u" ? null : self;
  return g(e) ? 0 : s(e);
}
__name(Jt, "Jt");
function Xt() {
  let e = typeof window > "u" ? null : window;
  return g(e) ? 0 : s(e);
}
__name(Xt, "Xt");
function Gt() {
  return w(function(e) {
    let t = i(e).text();
    return s(t);
  }, arguments);
}
__name(Gt, "Gt");
function Yt(e, t, n) {
  let r = i(e).then(i(t), i(n));
  return s(r);
}
__name(Yt, "Yt");
function Kt(e, t) {
  let n = i(e).then(i(t));
  return s(n);
}
__name(Kt, "Kt");
function Qt(e, t) {
  let n = i(t).url, r = A(n, _.__wbindgen_export, _.__wbindgen_export2), o = h;
  f().setInt32(e + 4, o, true), f().setInt32(e + 0, r, true);
}
__name(Qt, "Qt");
function Zt(e) {
  let t = i(e).view;
  return g(t) ? 0 : s(t);
}
__name(Zt, "Zt");
function te(e, t) {
  let n = le(e, t, ie);
  return s(n);
}
__name(te, "te");
function ee(e, t) {
  let n = I(e, t);
  return s(n);
}
__name(ee, "ee");
function ne(e) {
  let t = i(e);
  return s(t);
}
__name(ne, "ne");
function re(e) {
  p(e);
}
__name(re, "re");
function c() {
  if (B) {
    J();
    return;
  }
}
__name(c, "c");
function _e(e, t, n, r) {
  c(), _.__wasm_bindgen_func_elem_345(e, t, s(n), s(r));
}
__name(_e, "_e");
function ie(e, t, n) {
  try {
    let b = _.__wbindgen_add_to_stack_pointer(-16);
    c(), _.__wasm_bindgen_func_elem_314(b, e, t, s(n));
    var r = f().getInt32(b + 0, true), o = f().getInt32(b + 4, true);
    if (o) throw p(r);
  } finally {
    _.__wbindgen_add_to_stack_pointer(16);
  }
}
__name(ie, "ie");
var oe = ["bytes"];
var u = 0;
var se = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: e, instance: t }) => {
  t === u && _.__wbg_containerstartupoptions_free(e, 1);
});
var ce = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: e, instance: t }) => {
  t === u && _.__wbg_intounderlyingbytesource_free(e, 1);
});
var ue = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: e, instance: t }) => {
  t === u && _.__wbg_intounderlyingsink_free(e, 1);
});
var fe = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: e, instance: t }) => {
  t === u && _.__wbg_intounderlyingsource_free(e, 1);
});
var ae = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: e, instance: t }) => {
  t === u && _.__wbg_minifyconfig_free(e, 1);
});
var be = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: e, instance: t }) => {
  t === u && _.__wbg_r2range_free(e, 1);
});
function s(e) {
  x === d.length && d.push(d.length + 1);
  let t = x;
  return x = d[t], d[t] = e, t;
}
__name(s, "s");
var V = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry((e) => {
  e.instance === u && _.__wbindgen_export4(e.a, e.b);
});
function L(e) {
  let t = typeof e;
  if (t == "number" || t == "boolean" || e == null) return `${e}`;
  if (t == "string") return `"${e}"`;
  if (t == "symbol") {
    let o = e.description;
    return o == null ? "Symbol" : `Symbol(${o})`;
  }
  if (t == "function") {
    let o = e.name;
    return typeof o == "string" && o.length > 0 ? `Function(${o})` : "Function";
  }
  if (Array.isArray(e)) {
    let o = e.length, b = "[";
    o > 0 && (b += L(e[0]));
    for (let a = 1; a < o; a++) b += ", " + L(e[a]);
    return b += "]", b;
  }
  let n = /\[object ([^\]]+)\]/.exec(toString.call(e)), r;
  if (n && n.length > 1) r = n[1];
  else return toString.call(e);
  if (r == "Object") try {
    return "Object(" + JSON.stringify(e) + ")";
  } catch {
    return "Object";
  }
  return e instanceof Error ? `${e.name}: ${e.message}
${e.stack}` : r;
}
__name(L, "L");
function ge(e) {
  e < 1028 || (d[e] = x, x = e);
}
__name(ge, "ge");
function de(e, t) {
  e = e >>> 0;
  let n = f(), r = [];
  for (let o = e; o < e + 4 * t; o += 4) r.push(p(n.getUint32(o, true)));
  return r;
}
__name(de, "de");
function we(e, t) {
  return e = e >>> 0, j().subarray(e / 1, e / 1 + t);
}
__name(we, "we");
var m = null;
function f() {
  return (m === null || m.buffer.detached === true || m.buffer.detached === void 0 && m.buffer !== _.memory.buffer) && (m = new DataView(_.memory.buffer)), m;
}
__name(f, "f");
function I(e, t) {
  return ye(e >>> 0, t);
}
__name(I, "I");
var E = null;
function j() {
  return (E === null || E.byteLength === 0) && (E = new Uint8Array(_.memory.buffer)), E;
}
__name(j, "j");
function i(e) {
  return d[e];
}
__name(i, "i");
function w(e, t) {
  try {
    return e.apply(this, t);
  } catch (n) {
    _.__wbindgen_export3(s(n));
  }
}
__name(w, "w");
var d = new Array(1024).fill(void 0);
d.push(void 0, null, true, false);
var x = d.length;
function g(e) {
  return e == null;
}
__name(g, "g");
function le(e, t, n) {
  let r = { a: e, b: t, cnt: 1, instance: u }, o = /* @__PURE__ */ __name((...b) => {
    if (r.instance !== u) throw new Error("Cannot invoke closure from previous WASM instance");
    r.cnt++;
    let a = r.a;
    r.a = 0;
    try {
      return n(a, r.b, ...b);
    } finally {
      r.a = a, o._wbg_cb_unref();
    }
  }, "o");
  return o._wbg_cb_unref = () => {
    --r.cnt === 0 && (_.__wbindgen_export4(r.a, r.b), r.a = 0, V.unregister(r));
  }, V.register(o, r, r), o;
}
__name(le, "le");
function pe(e, t) {
  let n = t(e.length * 4, 4) >>> 0, r = f();
  for (let o = 0; o < e.length; o++) r.setUint32(n + 4 * o, s(e[o]), true);
  return h = e.length, n;
}
__name(pe, "pe");
function A(e, t, n) {
  if (n === void 0) {
    let l = W.encode(e), v = t(l.length, 1) >>> 0;
    return j().subarray(v, v + l.length).set(l), h = l.length, v;
  }
  let r = e.length, o = t(r, 1) >>> 0, b = j(), a = 0;
  for (; a < r; a++) {
    let l = e.charCodeAt(a);
    if (l > 127) break;
    b[o + a] = l;
  }
  if (a !== r) {
    a !== 0 && (e = e.slice(a)), o = n(o, r, r = a + e.length * 3, 1) >>> 0;
    let l = j().subarray(o + a, o + r), v = W.encodeInto(e, l);
    a += v.written, o = n(o, r, a, 1) >>> 0;
  }
  return h = a, o;
}
__name(A, "A");
var B = false;
function p(e) {
  let t = i(e);
  return ge(e), t;
}
__name(p, "p");
var z = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true });
z.decode();
var he = 2146435072;
var F = 0;
function ye(e, t) {
  return F += t, F >= he && (z = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true }), z.decode(), F = t), z.decode(j().subarray(e, e + t));
}
__name(ye, "ye");
var W = new TextEncoder();
"encodeInto" in W || (W.encodeInto = function(e, t) {
  let n = W.encode(e);
  return t.set(n), { read: e.length, written: n.length };
});
var h = 0;
var _;
function $(e) {
  _ = e;
}
__name($, "$");
var P = new WebAssembly.Instance(me, { "./index_bg.js": y, "./snippets/worker-64cc4b2d035528c7/inline0.js": D });
$(P.exports);
P.exports.__wbindgen_start?.();
var U = class extends xe {
  static {
    __name(this, "U");
  }
  async fetch(t) {
    return await C(t, this.env, this.ctx);
  }
  async queue(t) {
    return await (void 0)(t, this.env, this.ctx);
  }
  async scheduled(t) {
    return await (void 0)(t, this.env, this.ctx);
  }
};
var Ie = ["IntoUnderlyingByteSource", "IntoUnderlyingSink", "IntoUnderlyingSource", "MinifyConfig", "PolishConfig", "R2Range", "RequestRedirect", "fetch", "queue", "scheduled", "getMemory"];
Object.keys(y).map((e) => {
  Ie.includes(e) | e.startsWith("__") || (U.prototype[e] = y[e]);
});
var We = U;

// ../../../../../../../opt/homebrew/lib/node_modules/wrangler/templates/middleware/middleware-ensure-req-body-drained.ts
var drainBody = /* @__PURE__ */ __name(async (request, env, _ctx, middlewareCtx) => {
  try {
    return await middlewareCtx.next(request, env);
  } finally {
    try {
      if (request.body !== null && !request.bodyUsed) {
        const reader = request.body.getReader();
        while (!(await reader.read()).done) {
        }
      }
    } catch (e) {
      console.error("Failed to drain the unused request body.", e);
    }
  }
}, "drainBody");
var middleware_ensure_req_body_drained_default = drainBody;

// ../../../../../../../opt/homebrew/lib/node_modules/wrangler/templates/middleware/middleware-miniflare3-json-error.ts
function reduceError(e) {
  return {
    name: e?.name,
    message: e?.message ?? String(e),
    stack: e?.stack,
    cause: e?.cause === void 0 ? void 0 : reduceError(e.cause)
  };
}
__name(reduceError, "reduceError");
var jsonError = /* @__PURE__ */ __name(async (request, env, _ctx, middlewareCtx) => {
  try {
    return await middlewareCtx.next(request, env);
  } catch (e) {
    const error = reduceError(e);
    const body = JSON.stringify(error);
    const headers = {
      "Content-Type": "application/json",
      "MF-Experimental-Error-Stack": "true"
    };
    const encoded = encodeURIComponent(body);
    if (encoded.length <= 8192) {
      headers["MF-Experimental-Error-Stack-Payload"] = encoded;
    }
    return new Response(body, { status: 500, headers });
  }
}, "jsonError");
var middleware_miniflare3_json_error_default = jsonError;

// .wrangler/tmp/bundle-z23UcV/middleware-insertion-facade.js
var __INTERNAL_WRANGLER_MIDDLEWARE__ = [
  middleware_ensure_req_body_drained_default,
  middleware_miniflare3_json_error_default
];
var middleware_insertion_facade_default = We;

// ../../../../../../../opt/homebrew/lib/node_modules/wrangler/templates/middleware/common.ts
var __facade_middleware__ = [];
function __facade_register__(...args) {
  __facade_middleware__.push(...args.flat());
}
__name(__facade_register__, "__facade_register__");
function __facade_invokeChain__(request, env, ctx, dispatch, middlewareChain) {
  const [head, ...tail] = middlewareChain;
  const middlewareCtx = {
    dispatch,
    next(newRequest, newEnv) {
      return __facade_invokeChain__(newRequest, newEnv, ctx, dispatch, tail);
    }
  };
  return head(request, env, ctx, middlewareCtx);
}
__name(__facade_invokeChain__, "__facade_invokeChain__");
function __facade_invoke__(request, env, ctx, dispatch, finalMiddleware) {
  return __facade_invokeChain__(request, env, ctx, dispatch, [
    ...__facade_middleware__,
    finalMiddleware
  ]);
}
__name(__facade_invoke__, "__facade_invoke__");

// .wrangler/tmp/bundle-z23UcV/middleware-loader.entry.ts
var __Facade_ScheduledController__ = class ___Facade_ScheduledController__ {
  constructor(scheduledTime, cron, noRetry) {
    this.scheduledTime = scheduledTime;
    this.cron = cron;
    this.#noRetry = noRetry;
  }
  scheduledTime;
  cron;
  static {
    __name(this, "__Facade_ScheduledController__");
  }
  #noRetry;
  noRetry() {
    if (!(this instanceof ___Facade_ScheduledController__)) {
      throw new TypeError("Illegal invocation");
    }
    this.#noRetry();
  }
};
function wrapExportedHandler(worker) {
  if (__INTERNAL_WRANGLER_MIDDLEWARE__ === void 0 || __INTERNAL_WRANGLER_MIDDLEWARE__.length === 0) {
    return worker;
  }
  for (const middleware of __INTERNAL_WRANGLER_MIDDLEWARE__) {
    __facade_register__(middleware);
  }
  const fetchDispatcher = /* @__PURE__ */ __name(function(request, env, ctx) {
    if (worker.fetch === void 0) {
      throw new Error("Handler does not export a fetch() function.");
    }
    return worker.fetch(request, env, ctx);
  }, "fetchDispatcher");
  return {
    ...worker,
    fetch(request, env, ctx) {
      const dispatcher = /* @__PURE__ */ __name(function(type, init) {
        if (type === "scheduled" && worker.scheduled !== void 0) {
          const controller = new __Facade_ScheduledController__(
            Date.now(),
            init.cron ?? "",
            () => {
            }
          );
          return worker.scheduled(controller, env, ctx);
        }
      }, "dispatcher");
      return __facade_invoke__(request, env, ctx, dispatcher, fetchDispatcher);
    }
  };
}
__name(wrapExportedHandler, "wrapExportedHandler");
function wrapWorkerEntrypoint(klass) {
  if (__INTERNAL_WRANGLER_MIDDLEWARE__ === void 0 || __INTERNAL_WRANGLER_MIDDLEWARE__.length === 0) {
    return klass;
  }
  for (const middleware of __INTERNAL_WRANGLER_MIDDLEWARE__) {
    __facade_register__(middleware);
  }
  return class extends klass {
    #fetchDispatcher = /* @__PURE__ */ __name((request, env, ctx) => {
      this.env = env;
      this.ctx = ctx;
      if (super.fetch === void 0) {
        throw new Error("Entrypoint class does not define a fetch() function.");
      }
      return super.fetch(request);
    }, "#fetchDispatcher");
    #dispatcher = /* @__PURE__ */ __name((type, init) => {
      if (type === "scheduled" && super.scheduled !== void 0) {
        const controller = new __Facade_ScheduledController__(
          Date.now(),
          init.cron ?? "",
          () => {
          }
        );
        return super.scheduled(controller);
      }
    }, "#dispatcher");
    fetch(request) {
      return __facade_invoke__(
        request,
        this.env,
        this.ctx,
        this.#dispatcher,
        this.#fetchDispatcher
      );
    }
  };
}
__name(wrapWorkerEntrypoint, "wrapWorkerEntrypoint");
var WRAPPED_ENTRY;
if (typeof middleware_insertion_facade_default === "object") {
  WRAPPED_ENTRY = wrapExportedHandler(middleware_insertion_facade_default);
} else if (typeof middleware_insertion_facade_default === "function") {
  WRAPPED_ENTRY = wrapWorkerEntrypoint(middleware_insertion_facade_default);
}
var middleware_loader_entry_default = WRAPPED_ENTRY;
export {
  S as ContainerStartupOptions,
  k as MinifyConfig,
  R as R2Range,
  __INTERNAL_WRANGLER_MIDDLEWARE__,
  G as __wbg_String_8564e559799eccda,
  Y as __wbg___wbindgen_debug_string_4687d8d8c2017d52,
  K as __wbg___wbindgen_is_function_1f9d30630b8b1d3d,
  Q as __wbg___wbindgen_is_null_or_undefined_f9435d56bdcc06df,
  Z as __wbg___wbindgen_is_undefined_8865fb403f8fe9d8,
  tt as __wbg___wbindgen_reinit_3f79fdb34dee3ea9,
  et as __wbg___wbindgen_string_get_0380ccaa2f57f0d9,
  nt as __wbg___wbindgen_throw_41e9ee4f547fc59a,
  rt as __wbg__wbg_cb_unref_dcc1a90847f04c41,
  _t as __wbg_buffer_56ec2905a66f58b9,
  it as __wbg_byobRequest_54f7ce41d584549a,
  ot as __wbg_byteLength_9f985b8373f1f3fe,
  st as __wbg_byteOffset_0bd5a6a03553639c,
  ct as __wbg_call_187d372bd5fdd4aa,
  ut as __wbg_cause_19276d032a32d4bd,
  ft as __wbg_cf_b47fe5ff6d015a0d,
  at as __wbg_close_185286988f30f252,
  bt as __wbg_close_748f26698dabe269,
  gt as __wbg_enqueue_4a01c128f9a0de16,
  dt as __wbg_error_a30e98d44009f1bb,
  wt as __wbg_error_c9cf3fc2064683a9,
  lt as __wbg_get_88983e33467eae60,
  pt as __wbg_headers_6930a8bd09b630de,
  ht as __wbg_instanceId_9b67ff33ae8bef8a,
  yt as __wbg_instanceof_Error_80a725f81f2e102d,
  mt as __wbg_length_7f3c00c40364105e,
  xt as __wbg_message_5f8387f0c32b90a7,
  It as __wbg_method_6d02ee0808a7de31,
  vt as __wbg_name_e2eac7cdfa054f65,
  Et as __wbg_new_343a093a3c2ffb4e,
  jt as __wbg_new_617a8cdb8bb1130e,
  Ft as __wbg_new_83ca405eae412a62,
  Wt as __wbg_new_typed_b01cb72a8af741a3,
  St as __wbg_new_with_byte_offset_and_length_2f5d7fc2a828b74d,
  kt as __wbg_new_with_length_3da0ad195f6f63ba,
  Rt as __wbg_new_with_opt_buffer_source_and_init_c1b859a23c80afca,
  At as __wbg_new_with_opt_readable_stream_and_init_df114f2d364f216d,
  zt as __wbg_new_with_opt_str_and_init_33bef1b272950372,
  Ot as __wbg_queueMicrotask_9833f9a49df95a49,
  Tt as __wbg_queueMicrotask_a72f977e97f23c5f,
  Mt as __wbg_resolve_0076e10020304ede,
  Ut as __wbg_respond_c102fabcc79e5ef0,
  qt as __wbg_set_145a351398b48c65,
  Dt as __wbg_set_575d3ddb70fe831d,
  Lt as __wbg_set_af5932e3f78b5648,
  Ct as __wbg_set_criticalError_3904d60fe2a0b05b,
  Bt as __wbg_set_headers_69ef7c46a2e6c742,
  $t as __wbg_set_instanceId_9ad5caac0a4e519c,
  Nt as __wbg_set_status_678f122cbe435d95,
  $ as __wbg_set_wasm,
  Vt as __wbg_static_accessor_GLOBAL_266715b9d96ba635,
  Pt as __wbg_static_accessor_GLOBAL_THIS_10fb7dc1ae063179,
  Ht as __wbg_static_accessor_INIT_STATE_e3fce2ad8a08e94e,
  Jt as __wbg_static_accessor_SELF_0b583911f537483a,
  Xt as __wbg_static_accessor_WINDOW_d7f903d1508cbdc4,
  Gt as __wbg_text_aaf360cc4f1c0167,
  Yt as __wbg_then_c949d5a25a4e78f8,
  Kt as __wbg_then_e71170d78fcf8954,
  Qt as __wbg_url_7bfada5c2297bc70,
  Zt as __wbg_view_9c570f33e8d6ab96,
  te as __wbindgen_generic_0000000000000001,
  ee as __wbindgen_generic_0000000000000002,
  ne as __wbindgen_object_clone_ref,
  re as __wbindgen_object_drop_ref,
  X as __worker_init_state,
  middleware_loader_entry_default as default,
  C as fetch,
  me as wasmModule
};
//# sourceMappingURL=index.js.map
