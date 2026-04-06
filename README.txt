# Physics Sandbox

A real-time 2D rigid-body physics sandbox with client-server architecture, written in Rust.

The server runs the simulation; the client renders it with SDL2 and lets you interact with the physics world. Multiple clients can connect to the same server simultaneously.

![screenshot](assets/background.png)

---

## Features

- Circles and rectangles with mass, velocity, and rotation
- Gravity, damping, and restitution-based collision response
- Angular impulse and rotation for rectangles
- Motion trails (last 30 positions, fading alpha)
- Collision flash highlight effect
- Background texture rendering
- LAN multiplayer — connect multiple clients to one server

## Controls

| Input | Action |
|---|---|
| Left-click + drag on body, release | Apply impulse in drag direction |
| `R` | Spawn rectangle at mouse position |
| `C` | Spawn circle at mouse position |
| `Escape` / close window | Quit |

---

## Architecture

```
server (TCP :8080)
  └─ simulation loop @ ~60 Hz
  └─ broadcasts WorldState (JSON) to all clients
  └─ receives ClientMessage (JSON) from clients

client (SDL2 window 1200×800)
  └─ connects to server IP:port (prompted at startup)
  └─ renders bodies, trails, velocity vectors
  └─ sends user input as ClientMessage
```

Network protocol: newline-delimited JSON over raw TCP.

---

## Building

### Prerequisites

- Rust (stable) — https://rustup.rs
- SDL2 development libraries (already bundled in repo for Windows x64)

### Windows (x64)

```bat
cargo build --release
```

Binaries are output to `target/release/`.

---

## Running

### 1. Start the server

```bat
target\release\server.exe
```

The server listens on `0.0.0.0:8080`.

### 2. Start the client

```bat
target\release\client.exe
```

You will be prompted for the server IP and port. Press Enter to use defaults (`127.0.0.1:8080`).

### Quick launch (dev)

```bat
start.bat
```

---

## Release Package Layout

When distributing, place the following files together:

```
server.exe
client.exe
SDL2.dll
SDL2_image.dll
libavif-16.dll
libtiff-5.dll
libwebp-7.dll
libwebpdemux-2.dll
assets/
  background.png
```

All DLLs are included in the repository under the root directory and `optional/`.

---

## Dependencies

| Crate | Purpose |
|---|---|
| `sdl2` (0.35, `image` feature) | Rendering, input, window management |
| `serde` / `serde_json` | JSON serialization for network protocol |

---

## License

This library is distributed under the terms of the zlib license:

  This software is provided 'as-is', without any express or implied
  warranty.  In no event will the authors be held liable for any damages
  arising from the use of this software.

  Permission is granted to anyone to use this software for any purpose,
  including commercial applications, and to alter it and redistribute it
  freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not
     claim that you wrote the original software. If you use this software
     in a product, an acknowledgment in the product documentation would be
     appreciated but is not required.
  2. Altered source versions must be plainly marked as such, and must not be
     misrepresented as being the original software.
  3. This notice may not be removed or altered from any source distribution.

The source is available from the SDL website:
http://www.libsdl.org/projects/SDL_image
