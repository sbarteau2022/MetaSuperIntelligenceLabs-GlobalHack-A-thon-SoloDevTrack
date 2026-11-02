import { Suspense, lazy, useEffect, useRef } from 'react'
import { useReducedMotion } from '../../hooks/useReducedMotion'
import HeroContent from './HeroContent'

const HeroCanvas = lazy(() => import('./HeroCanvas'))

// Hero entrance: pure CSS keyframes, no gsap timeline — the gsap chunk
// stays out of the critical path and HeroCanvas (three.js) loads async.
export default function Hero() {
  const sectionRef = useRef<HTMLElement>(null)
  const reduced = useReducedMotion()

  useEffect(() => {
    if (reduced) return
    const scope = sectionRef.current
    if (!scope) return
    const els = Array.from(scope.querySelectorAll<HTMLElement>('[data-hero-rise]'))
    const io = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue
          const el = entry.target as HTMLElement
          io.unobserve(el)
          el.classList.add('hero-rise-in')
        }
      },
      { threshold: 0.1 },
    )
    for (const el of els) io.observe(el)
    return () => io.disconnect()
  }, [reduced ])

  return (
    <section ref={sectionRef} className="relative min-h-screen flex items-center overflow-hidden">
      {/* Hairline grid backdrop + radial vignette */}
      <div className="hairline-grid absolute inset-0" aria-hidden="true" />
      <div
        className="absolute inset-0 pointer-events-none"
        aria-hidden="true"
        style={{
          background:
            'radial-gradient(ellipse 80% 60% at 50% 40%, transparent 30%, rgba(2,4,8,0.75) 100%)',
        }}
      />

      {/* 3D canvas backdrop */}
      <div className="absolute inset-0 opacity-90" aria-hidden="true">
        <Suspense fallback={null}>
          <HeroCanvas />
        </Suspense>
      </div>

      {/* Content */}
      <div className="max-w-[1200px] mx-auto px-6 py-32 relative z-10 w-full text-left">
        <HeroContent />
      </div>

      {/* Hero entrance keyframes — CSS-only, no JS animation lib on critical path */}
      <style>{`
        [data-hero-rise] { opacity: 0; transform: translateY(26px); }
        @media (prefers-reduced-motion: reduce) {
          [data-hero-rise] { opacity: 1; transform: none; }
        }
        [data-hero-rise].hero-rise-in {
          animation: hero-rise 0.7s cubic-bezier(0.22,1,0.36,1) forwards;
        }
        @keyframes hero-rise { to { opacity: 1; transform: translateY(0); } }
      `}</style>
    </section>
  )
}
