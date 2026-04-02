# Kemari: Ritual Play (Rust + Bevy)

A small Rust game prototype inspired by **kemari**, a Heian-era Japanese court ball game.  
This project is focused on **cooperative “keep it up” play**, ritual structure, and “serious play” rather than competition.

The goal is a **vertical slice**: one court, one ball, one controllable player, a few NPCs, a chain counter, and an “elegance” meter.

---

## What the game is

Kemari isn’t about winning. It’s about maintaining the ball in the air with control, rhythm, and social/ritual correctness.

In this prototype you:
- move on a square court
- time kicks to keep the ball aloft
- build a **chain counter**
- build (or lose) **elegance** based on how you play

---

## Core loop

1. Position under/near the falling ball  
2. Time a kick (not spam)  
3. Ball stays up → chain increases  
4. Elegant play increases your elegance meter  
5. Messy play drops elegance (and sometimes your chain)

---

## Controls (planned / current)

- WASD: move
- Space: kick
- R: reset game (score, ball, character placements)
- Esc: quit

---

## MVP scope (vertical slice)

**Must-have:**
- 1 court scene
- 1 ball with basic gravity/physics
- 1 playable character + kick action
- 7 NPCs (static/idle visuals to sell “group play”)
- chain counter UI
- elegance meter UI (rewards controlled play)

**Nice-to-have / stretch:**
- pass-to-NPC behavior (simple assist AI)
- ritual “rules” that reward specific patterns (variety, calm pacing)
- tree corners / environment influence (visual + slight gameplay effect)
- audio + polish + small tutorial

---

## Design pillars

- **Cooperation over competition**: no “win/lose,” focus on sustaining play.
- **Ritual structure**: repetition, rhythm, and form matter.
- **Elegance as feedback**: score isn’t just “hits,” it’s *how* you play.

See: `docs/01_design_pillars.md`

---

## Tech

- Language: Rust
- Engine: Bevy
- Assets: placeholder shapes/sprites for now

---

## Asset Credits

- "Sakura Tree 01 - Low Poly Model" by Jogoss
  License: CC BY 4.0
  Source: https://sketchfab.com/3d-models/sakura-tree-01-low-poly-model-147ae7d0d332456a99ec6195e9b0cd4f?utm_source=chatgpt.com
  Changes: Converted/used in-game, scaled/rotated/compressed as needed

---

## How to run

```bash
cargo run
