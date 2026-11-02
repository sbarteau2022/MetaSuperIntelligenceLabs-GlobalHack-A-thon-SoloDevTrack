---
name: marvel-architect-scribe
description: >-
  Generates epic, first-person, highly philosophical and deeply mathematical architecture whitepapers and slide decks. Blends Marvel-like storytelling ("Sean Carroll meets Tony Stark") with peer-reviewed academic rigor and exact DOIs from OpenAlex.
---

# Marvel Architect Scribe

## Overview
This skill distills the workflow of creating the ultimate "Atlas" style architecture whitepaper. It combines a highly accessible, narrative-driven front-end (Möbius Inversion) with a bulletproof, mathematically rigorous back-end. It actively queries OpenAlex for peer-reviewed citations, embeds the equations using LaTeX, and formats everything as a presentation-ready markdown artifact with generated cover images.

## Dependencies
- `literature-search-openalex` (Required for pulling accurate DOIs and papers).
- `generate_image` (Required for creating cinematic, high-fidelity cover concepts).

## Workflow

### 1. The Philosophical Front-End
- Adopt a first-person, narrative-heavy voice ("Sean Carroll meets Tony Stark").
- Explain the problem (e.g., stochastic parrots, floating point errors) with metaphor and visceral imagery.
- Introduce the solution as a profound geometrical or architectural shift (e.g., Dual Torus, Poincaré disk, Q0.30 fixed-point).
- Ensure the prose is epic, beautiful, and "Marvel-like."

### 2. The Academic Back-End
- Pivot to rigorous, bulletproof mathematics.
- Break down the architecture into exact equations (Symplectic decoding, KAM theory, Friston's Free Energy Principle).
- Review all LaTeX math for correctness and formatting.

### 3. Literature Search & Citations
- Use `literature-search-openalex` to pull exact DOIs for all claims.
- Create an Appendix that strictly details the mathematical breakdown of each cited paper and how it applies to the architecture.

### 4. Cover Art Generation
- Use `generate_image` to create a cinematic cover image (e.g., "A highly detailed, cinematic visualization of a Dual Torus embedded within a glowing Poincare disk...").
- Embed the image path directly into the markdown artifact.

### 5. Final Assembly
- Output the document into the requested local repository as a `.md` file, partitioned with `---` slide breakpoints for easy visualization.
