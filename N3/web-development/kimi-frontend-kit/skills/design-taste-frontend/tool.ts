class AntiSlop {
  scroll_listener: boolean;
  images_real: boolean;
  status_dots: string;
  footer_version: boolean;
  constructor(scroll_listener: boolean, images_real: boolean, status_dots: string, footer_version: boolean) {
    this.scroll_listener = scroll_listener;
    this.images_real = images_real;
    this.status_dots = status_dots;
    this.footer_version = footer_version;
  }
}
function scroll_ok(scroll_listener: boolean): boolean {
  if (scroll_listener) {
    return false;
  }
  return true;
}
function dots_ok(status_dots: string): boolean {
  if (status_dots === "none") {
    return true;
  }
  if (status_dots === "semantic-single") {
    return true;
  }
  return false;
}
function antislop_gate(cfg: AntiSlop): boolean {
  if (scroll_ok(cfg.scroll_listener)) {
    if (cfg.images_real) {
      if (dots_ok(cfg.status_dots)) {
        if (cfg.footer_version) {
          return false;
        }
        return true;
      }
      return false;
    }
    return false;
  }
  return false;
}
function make_antislop(scroll_listener: boolean, images_real: boolean, status_dots: string, footer_version: boolean): AntiSlop {
  return new AntiSlop(scroll_listener, images_real, status_dots, footer_version);
}
