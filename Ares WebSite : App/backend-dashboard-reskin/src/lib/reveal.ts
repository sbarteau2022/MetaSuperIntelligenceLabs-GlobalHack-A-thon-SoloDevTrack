/**
 * Reveal-on-scroll driver (IntersectionObserver + CSS — no animation library).
 * Elements tagged [data-reveal] fade+rise once when entering the viewport.
 * Honors prefers-reduced-motion (animations become instant).
 * Returns a cleanup function — call it from the effect that invoked it.
 */
export function revealOnScroll(scope: HTMLElement): () => void {
  const els = Array.from(scope.querySelectorAll<HTMLElement>('[data-reveal]'))
  // Include the scope itself if tagged.
  if (scope.hasAttribute('data-reveal')) els.push(scope)

  const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches

  for (const el of els) {
    // Cheap initial state via inline style (no gsap.set per element).
    el.style.opacity = '0'
    el.style.transform = 'translateY(28px)'
    el.style.willChange = 'opacity, transform'
  }

  if (reduced) {
    for (const el of els) {
      el.style.opacity = '1'
      el.style.transform = 'none'
      el.style.willChange = 'auto'
    }
    return () => {}
  }

  const io = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue
        const el = entry.target as HTMLElement
        io.unobserve(el)
        const delay = parseFloat(el.dataset.delay || '0')
        el.style.transition = `opacity 0.7s cubic-bezier(0.22,1,0.36,1) ${delay}s, transform 0.7s cubic-bezier(0.22,1,0.36,1) ${delay}s`
        // Double rAF so the transition picks up the initial state.
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            el.style.opacity = '1'
            el.style.transform = 'translateY(0)'
          })
        })
        // Clean up will-change after the animation settles.
        window.setTimeout(() => {
          el.style.willChange = 'auto'
        }, (delay + 0.8) * 1000)
      }
    },
    { rootMargin: '0px 0px -12% 0px', threshold: 0.05 },
  )

  for (const el of els) io.observe(el)
  return () => io.disconnect()
}
