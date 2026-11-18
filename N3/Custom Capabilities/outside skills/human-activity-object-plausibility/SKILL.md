---
name: human-activity-object-plausibility
description: >-
  Expert rubric, decision trees, and calibrated benchmark guidelines for evaluating
  AI-generated human activity videos (Feather, OpenAI directives rollout, r4 single-video
  object plausibility, and legacy pairwise human activity quality). Use whenever grading
  directive task completion, physical plausibility (shape/appearance, penetration, motion),
  or analyzing video screener / golden tasks.
---

# Human Activity Object Plausibility & Directive Auditing

## Core Principles
1. **Independent Scoring:** Directive Completion and Physical Plausibility are scored strictly independently. A visually awkward or failing action can be physically plausible; an apparent success can violate physics.
2. **Independent Axes:** Score Shape/Appearance, No Penetration, and Motion separately. One clear, meaningful failure on an axis warrants a Bad rating. Do not copy answers across axes without distinct evidence.
3. **Visible Evidence Only:** Judge what is visible and within the named target's scope. Do not assume hidden offscreen events or substitute nearby objects.
4. **Prioritize Unmistakable Issues:** Focus on primary actions and interacting targets. Ordinary motion blur, sensor noise, or ambiguous pixel boundaries alone are never physical failures.

---

## 1. Pre-Screening Gates

| Status | Trigger Condition | Required Action |
| :--- | :--- | :--- |
| **Cannot Evaluate** | • Entire frame or focal action area becomes completely unrecognizable, even momentarily.<br>• Hard shot cut or scene change breaks continuation (~0.3s continuity thought experiment). | Stop evaluation. Provide mandatory brief reason. |
| **Still Evaluate** (Proceed) | • Ordinary motion blur, low light, or sensor noise where objects/actions remain identifiable.<br>• Minor local jump cuts or teleports within a single continuing shot. | Proceed with evaluation. Score physical issues under the relevant question; blur alone is not a failure. |
| **Not Present at All** | • The specific named target object never appears at any moment throughout the entire clip. | Skip physical axes for this target only. Still score Task Completion and any second target. |
| **Temporary Occlusion / Mid-Video Exit** | • Object appears initially, then leaves frame, is blocked by person/furniture, or inexplicably vanishes. | Do NOT mark "Not Present". Evaluate normally. Vanishing is graded under Shape/Appearance. |

---

## 2. Question 1: Directive / Task Completion

### Rule on Starting State
If the video starts mid-action (e.g., person is already reaching, leaning, or holding the object), **credit earlier steps as completed**. Never penalize for actions completed off-screen before the clip started. Grade only remaining actions and lasting outcomes.

| Level | Standard Definition | Benchmark Reference |
| :--- | :--- | :--- |
| **0 — none** | No meaningful remaining action or useful progress is achieved toward the goal. | Person remains stationary; hands do not move toward the target. |
| **About one-third** | A small but meaningful initial part is achieved; the vast majority remains undone. | Reaches and touches the cabinet handle, but stops without opening it. |
| **About two-thirds** | Substantial progress achieved, but an important action or lasting outcome is missing. | Cabinet is swung open exposing items, but clip ends before reaching in and retrieving target; or starts walking across room toward stove, but video cuts off before reaching destination. |
| **Fully achieved** | All meaningful remaining actions and required lasting outcomes are completely achieved. | Lifts bag into lap and steadies it; required end-state holds firmly at video end. |

---

## 3. Physical Plausibility Axes (Target-Level)

### Axis 1: Shape and Appearance
* **Rigid Objects:** Must maintain structural rigidity. Mark **Bad** if rigid plastic, metal, wood, or glass bends, squashes, stretches, collapses, or melts without external force.
* **Soft Materials & Liquids:** Bending, folding, wrinkling, and flowing are expected. Mark **Bad** if fabric patterns disintegrate into melted blotches, or materials dissolve/morph unnaturally into another object.
* **Integrity & Identity:** Mark **Bad** if parts spontaneously duplicate, develop phantom holes, fuse, or turn into another object.
* **Unexplained Disappearance:** If an object or visible part inexplicably vanishes, rate **Bad**. Normal occlusion, leaving view, or expected consumption is Good.
* **Mark Good:** Perspective changes, shadows, lighting shifts, temporary occlusions, or subtle texture flicker.

### Axis 2: No Penetration
* **Standard:** Solid surfaces must not pass through other solid surfaces or occupy identical 3D space.
* **Mark Bad:** Unmistakable structural crossing (e.g., a handle slicing through a wrist/forearm, an object sinking through a tabletop, bag clipping through desk edge).
* **Mark Good:** Ambiguous hand/contact boundaries due to motion blur, shortened fingers from camera angles, normal compression of soft cushions, or entering natural openings.
* **Key Distinction:** Two objects merging or an internal shape morph is a **Shape failure**, not penetration.

### Axis 3: Motion
* **Standard:** Movement must follow gravity, support surfaces, momentum, friction, and kinematics.
* **Mark Bad:** Sudden impossible jumps/teleportation, floating without support, unprompted acceleration without contact, or rigid joints bending against design (e.g. hinged door pulling straight out like a drawer).
* **Mark Good:** Objects remaining stationary at rest (even if directive requested pickup), realistic drops/slips, and subtle settling/compression.
* **Grip Tolerance:** If hand placement is reasonably close to an intended grasp, credit the grip and grade whether the object's path matches hand motion.

---

## 4. Confusions Between Questions & Critical Rules

### The 3-Step Disappearance Rule
1. If the target inexplicably vanishes: Rate **Shape and Appearance: Bad**.
2. Score **Penetration and Motion only on the frames before** the disappearance occurs.
3. **Do not automatically mark Penetration or Motion Bad purely because it vanished.**

### Shape vs. Penetration
* An object's own internal shape morphing is **not** penetration. (e.g., a pair of shoes fusing into one shoe-like form is **Shape Bad**, **Penetration Good**).
* If two distinct starting objects merge into each other, mark **No Penetration Bad** (and Shape Bad if forms deform).

### Shape vs. Motion
* An implausible shape morph alone is **not** a motion error. (e.g., a rigid can stretching into a tapered form as it is lowered: **Shape Bad**, **Motion Good** because its path remains a continuous hand-led movement).
* Constraint violations (e.g. hinged door pulling out like a drawer) are **Motion Bad**, not Shape issues.

---

## 5. Legacy Pairwise Rubric Mapping
* **Q1 (Progress):** Overall completion rate of all instructed actions across videos.
* **Q2 (Human Action/Appearance Plausibility):** Covers changes in the human only (limbs, torso, face, gestures). Object deformations or vanishing do **NOT** belong here.
* **Q3 (Object & Interaction Plausibility):** Covers interacting objects (geometry, motion, vanishing) and causal interaction realism (forces, collisions).
* **Q4 (Visual Effects & Clarity):** Artifacts, flickering, rendering clarity.
