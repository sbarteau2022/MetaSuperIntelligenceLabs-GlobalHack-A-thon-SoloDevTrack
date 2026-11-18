export default function HairlineGrid() {
  return (
    <div
      aria-hidden="true"
      className="hairline-grid pointer-events-none absolute inset-0"
      style={{
        maskImage:
          'radial-gradient(ellipse 90% 80% at 50% 20%, black 30%, transparent 75%)',
        WebkitMaskImage:
          'radial-gradient(ellipse 90% 80% at 50% 20%, black 30%, transparent 75%)',
      }}
    />
  )
}
