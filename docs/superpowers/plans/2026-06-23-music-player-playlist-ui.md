# Music Player Playlist UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refresh the music player playlist popover into a light, card-based music app layout with clearer hierarchy and stronger current-track emphasis.

**Architecture:** Keep the existing `MusicPlayer` structure and `Popover` flow intact, and restyle only the playlist popover and item rows inside `src/drive/music_player/ui.rs`. Reuse current player state to drive a featured header, card rows, and the playing-state badge without introducing new state or new modules.

**Tech Stack:** Rust, gpui, gpui-component, existing `MusicPlayer` view helpers

---

### Task 1: Define the visual structure for the playlist popover

**Files:**
- Modify: `src/drive/music_player/ui.rs`
- Verify: `cargo check`

- [ ] **Step 1: Inspect the current playlist popover and row renderer**

Read the `player_list_vm` and `player_list_ui` functions in `src/drive/music_player/ui.rs` and confirm that the current implementation uses a plain list row inside a `Popover`.

- [ ] **Step 2: Replace the popover container styling with a card-based shell**

Update the `player_list_ui` popover content to include:

```rust
div()
    .w(px(760.))
    .h(px(560.))
    .rounded_xl()
    .border_1()
    .border_color(rgb_to_u32(226, 232, 240))
    .bg(rgb_to_u32(255, 255, 255))
    .shadow_lg()
    .overflow_hidden()
```

and a vertical layout with:

```rust
v_flex()
    .size_full()
    .gap_3()
    .p_4()
```

- [ ] **Step 3: Add a lightweight header and current-track summary**

Add a top section that shows:

```rust
div().child("Playlist")
div().child(format!("{} tracks", self.player_list.len()))
```

and a current-track summary card that reads from `self.current_player`, with a fallback title when no song is selected.

- [ ] **Step 4: Run compile verification after the shell refactor**

Run: `cargo check`
Expected: exit code `0`

### Task 2: Convert each playlist row into a music-style mini card

**Files:**
- Modify: `src/drive/music_player/ui.rs`
- Verify: `cargo check`

- [ ] **Step 1: Reshape the row sizing for card items**

Change the virtual list row size from:

```rust
.map(|_| size(px(100.), px(40.)))
```

to a larger card height such as:

```rust
.map(|_| size(px(100.), px(88.)))
```

- [ ] **Step 2: Replace the plain row body with a rounded card row**

Render each item as:

```rust
div()
    .w_full()
    .px_2()
    .py_1()
    .child(
        h_flex()
            .w_full()
            .items_center()
            .justify_between()
            .gap_3()
            .p_3()
            .rounded_lg()
            .border_1()
```

with conditional background and border colors based on whether the item is the current track.

- [ ] **Step 3: Add cover art, title metadata, and source metadata**

Use a larger rounded cover image with a fallback placeholder when `data.img` is empty, plus a text stack for:

```rust
data.name
data.author
data.source
```

where title is stronger and metadata is smaller/lighter.

- [ ] **Step 4: Upgrade the action area**

Keep the existing playback behavior, but render:
- a status badge such as `Playing` for the current row
- an outlined or filled pill-style `Play` button for other rows

- [ ] **Step 5: Run compile verification after the row refactor**

Run: `cargo check`
Expected: exit code `0`

### Task 3: Tune spacing, scrolling, and motion for a cleaner light UI

**Files:**
- Modify: `src/drive/music_player/ui.rs`
- Verify: `cargo check`

- [ ] **Step 1: Adjust scroll area spacing**

Wrap the virtual list section in a rounded light surface and keep the existing scrollbar:

```rust
div()
    .flex_1()
    .rounded_lg()
    .bg(rgb_to_u32(248, 250, 252))
    .border_1()
```

- [ ] **Step 2: Keep the popover animation while matching the new shell size**

Update the `with_animation` height interpolation to match the new popover height so the entrance still feels like a soft upward reveal.

- [ ] **Step 3: Run final verification**

Run: `cargo check`
Expected: exit code `0`
