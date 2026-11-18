import { config } from '../lib/config';

export interface SeoProps {
  title: string;
  description: string;
  path: string;
  image?: string;
  noindex?: boolean;
}

/** Single head manager — every route declares its contract here. */
export function applySeo({ title, description, path, image, noindex }: SeoProps) {
  const url = `${config.siteUrl.replace(/\/$/, '')}${path.startsWith('/') ? path : `/${path}`}`;
  document.title = `${title} — Atoma Engineering`;
  const set = (selector: string, attr: 'content' | 'href', value: string) => {
    let el = document.head.querySelector(selector) as HTMLMetaElement | HTMLLinkElement | null;
    if (!el) {
      el = selector.startsWith('link')
        ? document.createElement('link')
        : (document.createElement('meta') as HTMLMetaElement);
      if (selector.startsWith('link')) {
        const rel = selector.match(/rel="([^"]+)"/)?.[1];
        if (rel) (el as HTMLLinkElement).rel = rel;
      } else {
        const key = selector.match(/(name|property)="([^"]+)"/);
        if (key) el.setAttribute(key[1], key[2]);
      }
      document.head.appendChild(el);
    }
    el.setAttribute(attr, value);
  };
  set('meta[name="description"]', 'content', description);
  set('meta[property="og:title"]', 'content', title);
  set('meta[property="og:description"]', 'content', description);
  set('meta[property="og:url"]', 'content', url);
  if (image) set('meta[property="og:image"]', 'content', image);
  set('link[rel="canonical"]', 'href', url);
  let robots = document.head.querySelector('meta[name="robots"]') as HTMLMetaElement | null;
  if (noindex) {
    if (!robots) {
      robots = document.createElement('meta');
      robots.name = 'robots';
      document.head.appendChild(robots);
    }
    robots.content = 'noindex, nofollow';
  } else if (robots) {
    robots.remove();
  }
}
