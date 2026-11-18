import React from 'react';
import { useAtomaStore } from '../../store/useAtomaStore';
import { clearUnknownRoute } from '../../lib/router';

export const NotFound: React.FC = () => {
  const missing = (() => {
    try {
      return sessionStorage.getItem('atoma.missing-route') ?? '';
    } catch {
      return '';
    }
  })();
  const { setActivePage } = useAtomaStore();

  const go = (page: Parameters<typeof setActivePage>[0]) => {
    clearUnknownRoute();
    setActivePage(page);
    window.scrollTo({ top: 0 });
  };

  return (
    <main
      className="max-w-[760px] mx-auto px-6 py-24 text-center"
      role="main"
      aria-labelledby="nf-title"
    >
      <div className="font-mono text-xs uppercase tracking-[0.25em] text-[#f59e0b]">
        404 · Route not charted
      </div>
      <h1 id="nf-title" className="mt-3 text-4xl sm:text-5xl font-bold text-white tracking-tight">
        That sector doesn&apos;t exist.
      </h1>
      <p className="mt-4 text-[#71717a] leading-relaxed">
        {missing ? (
          <>No lane matches <code className="font-mono text-xs text-white bg-white/8 px-1.5 py-0.5 rounded">{missing}</code>. </>
        ) : null}
        The platform held position — pick a known lane below.
      </p>
      <div className="mt-8 flex flex-wrap justify-center gap-3">
        <button onClick={() => go('platform')} className="px-5 py-2.5 rounded-lg bg-[#f59e0b] text-white text-sm font-semibold hover:bg-[#d97706] transition cursor-pointer">Platform home</button>
        <button onClick={() => go('ide-workbench')} className="px-5 py-2.5 rounded-lg bg-white/8 border border-white/15 text-sm text-white hover:bg-white/12 transition cursor-pointer">IDE &amp; DAG Lab</button>
        <button onClick={() => go('cloud-app')} className="px-5 py-2.5 rounded-lg bg-white/8 border border-white/15 text-sm text-white hover:bg-white/12 transition cursor-pointer">Cloud workspace</button>
      </div>
    </main>
  );
};
