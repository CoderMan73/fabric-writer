# Blue Nether Development Guide

This document describes the Blue Nether Mod development workflow for
`fabric-writer`. The full content specification is in `blue_nether_spec.md`.

## Current Implementation Status

| Step | Name | State |
|---|---|---|
| Step 1 | The Blue Nether Dimension | PARTIAL |
| Step 2 | Blue Obsidian and Blue Nether Portal | PARTIAL |
| Step 3 | Blue Nether Base Terrain | DONE |
| Step 4 | Blue Nether Biomes | PARTIAL |
| Step 5 | Blue Nether Blocks | DONE |
| Step 6 | Blue Nether Items | DONE |
| Step 7 | Blue Nether Mobs | PARTIAL |
| Step 8 | Blue Nether Structures | PARTIAL |
| Step 9 | Blue Nether Terrain Features | PARTIAL |
| Step 10 | Blue Fog, Particles, and Lighting | TODO |
| Step 11 | Blue Nether Flora and Vegetation | DONE |
| Step 12 | Blue Nether Audio | PARTIAL |
| Step 13 | Blue Nether Loot and Economy | PARTIAL |
| Step 14 | Blue Nether Recipes | DONE |
| Step 15 | Advancement and Achievement Integration | PARTIAL |

**State legend:** `DONE` — all content generated as valid JSON via datagen and committed to state; `PARTIAL` — content registered but functionality is stubbed or incomplete; `TODO` — not yet implemented.

## Preset Workflow

The Blue Nether preset is generated via:

```bash
fw test-project --preset blue-nether-base --reset
```

This single command:
1. Loads mod state from `.fw/fabric-writer.yml`
2. Resets all tracked entity collections (if `--reset`)
3. Adds all Blue Nether content entities to state (blocks, items, mobs, recipes, biomes, dimensions, structures, features, sound events, loot tables, advancements, tags, creative tabs)
4. Calls `state.save()` to persist the YAML
5. Calls `regenerate_all()` to emit all 21 Java files + resource JSONs
6. Copies and tints vanilla textures from `E:\Coding_Projects\MCSourceCode\vanilla-minecraft\26.2\assets\minecraft\textures`

**Critical:** All `state.add()` calls must complete before `state.save()` and
`regenerate_all()` — insertion order matters for entity ID assignment.

## Texture Tinting Pipeline

The `tint_texture()` function in `src/commands/test_project.rs` performs build-time
texture processing:

1. Copies vanilla texture PNG from `E:\Coding_Projects\MCSourceCode\vanilla-minecraft\26.2\assets\minecraft\textures`
2. Converts to grayscale via Rec. 601 luminance: `gray = 0.30*R + 0.59*G + 0.11*B`
3. Tints to blue palette color `#5078FF` with: `result[i] = gray[i] * tint[i] / 255`
4. Preserves RGBA transparency (e.g., `netherite_ingot` retains 121/256 transparent pixels)

## Incremental Execution Protocol

**Workflow sequence** (must be followed in order for each iteration):

```
Step 1: cargo run -- run d          # Run datagen — generates all JSON/resources
                                          # Review the blue_nether_test.log for:
                                          #   - Texture integrity (16×16 RGBA)
                                          #   - Model JSON validity
                                          #   - Java file presence
                                          #   - Critical errors during datagen
                                          #   - "Missing block model" warnings
                                          # If any FAIL or ERROR: fix root cause, re-run Step 1

Step 2: cargo run -- run dc          # Run datagen + client — launches Minecraft
                                          # This requires manual user intervention (MC client)
                                          # Capture in-game diagnostics:
                                          #   - Textures visible in inventory/creative tab
                                          #   - Dimension loads correctly
                                          #   - Blocks render with correct models
                                          #   - No shader/GL errors in log (fw_datagen.log)
                                          # Hand off to user for this step

Step 3: testing/test-blue-nether.ps1 # Automated validation script
                                          # Runs all test suites:
                                          #   - Preset regeneration
                                          #   - Texture pipeline output format
                                          #   - Texture integrity (Python/Pillow)
                                          #   - Model JSON validity
                                          #   - Java file presence
                                          #   - State file validation
                                          #   - Datagen + client build success
                                          #   - Missing model/sound warnings
                                          #   - Texture transparency preservation
                                          #   - Dimension loading verification
                                          # Generates blue_nether_test.log with all findings
                                          # If any ERROR-level items: return to Step 1
```

**Gate criteria for advancing between steps:**

| Step | Exit criteria |
|---|---|
| Step 1 → Step 2 | `blue_nether_test.log` shows no ERROR-level items; all PASS checks pass; texture integrity valid; Java files present; state file contains all expected entities |
| Step 2 → Step 3 | Minecraft client launches without crash; Blue Nether dimension loads (confirmed in `fw_datagen.log`); all block/item textures render (no purple/black checkerboard); no GL errors |
| Step 3 → Done | `test-blue-nether.ps1` summary shows all critical checks PASS; no ERROR or WARN items beyond known issues (placeholder sound warnings, creative tab icon warning) |

**Known limitations:**
- Step 2 (`cargo run -- run dc`) requires the user to manually launch and inspect the Minecraft client
- The profile key pair auth error (HTTP 401) is an account issue and cannot be resolved in code
- The vanilla sound source directory (`mc-sounds/26.2/`) does not currently exist; placeholder `empty.ogg` files are used instead

## Remaining Work Queue

| Priority | Task | Spec Step |
|---|---|---|
| P1 | Fix creative tab icon model warning (texture rendering resolution) | 3.1 |
| P2 | Add remaining biomes with MobSpawn rules | Step 4 |
| P3 | Add terrain feature codegen (configured/placed features) | Step 9 |
| P4 | Add mob spawn rules, AI, and drop tables | Step 7 |
| P5 | Add remaining 9 advancements | Step 15 |
| P6 | Add dimension type JSON and portal ignition logic | Steps 1-2 |
| P7 | Integrate real vanilla sound files | Step 12 |
| P8 | Add structure schematics (fortress, bastion) | Step 8 |
