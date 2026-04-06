# physics_sandbox.rs

**English** | [中文](#中文说明)

A real-time 2D rigid-body physics sandbox with LAN multiplayer, written in Rust.

The server runs the physics simulation at 60 Hz and broadcasts world state to all connected clients over TCP. The client renders everything with SDL2 and lets you interact with the world in real time.

---

## Features

**Physics**
- Circles and rectangles with mass, velocity, and rotation
- Gravity, linear & angular damping
- AABB collision detection with impulse resolution
- Angular impulse for rotating rectangles on collision
- Elastic collisions (restitution = 0.8)

**Rendering**
- 1200×800 SDL2 window, 60 FPS vsync
- Background texture
- Bodies colored by mass — red = heavy, blue/green = light
- Motion trails (last 30 positions, fading alpha)
- Collision flash highlight (yellow outline)
- Velocity vector from each body's center

**Multiplayer**
- Multiple clients can connect to the same server simultaneously
- Full world state broadcast every tick

---

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
server  (TCP 0.0.0.0:8080)
  ├─ simulation loop @ 60 Hz
  ├─ broadcasts WorldState JSON to all clients each tick
  └─ receives ClientMessage JSON from clients

client  (SDL2 1200×800)
  ├─ prompts for server IP:port at startup
  ├─ background thread receives WorldState
  └─ main thread renders + sends user input
```

Protocol: newline-delimited JSON over raw TCP.

---

## Building

**Prerequisites:** [Rust (stable)](https://rustup.rs) — SDL2 is already bundled for Windows x64.

```powershell
cargo build --release
```

Binaries output to `target/release/`.

---

## Running

```powershell
# 1. Start the server
.\target\release\server.exe

# 2. Start the client (prompts for IP:port, default 127.0.0.1:8080)
.\target\release\client.exe
```

Or use `start.bat` for a quick dev launch.

---

## Release Package

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

All DLLs are included in the repo.

---

## Dependencies

| Crate | Use |
|---|---|
| `sdl2` 0.35 (`image` feature) | Rendering, input, window |
| `serde` / `serde_json` | JSON network protocol |

---

## License

MIT

---

# 中文说明

**[English](#physic_sandboxrs)** | 中文

用 Rust 编写的实时 2D 刚体物理沙盒，支持局域网联机。

服务器以 60 Hz 运行物理模拟，通过 TCP 向所有连接的客户端广播世界状态。客户端用 SDL2 渲染画面，并允许你实时与物理世界交互。

---

## 功能特性

**物理**
- 圆形和矩形，具有质量、速度和旋转属性
- 重力、线速度阻尼、角速度阻尼
- AABB 碰撞检测 + 冲量解算
- 矩形碰撞时产生角冲量（真实旋转效果）
- 弹性碰撞（恢复系数 0.8）

**渲染**
- 1200×800 SDL2 窗口，60 FPS 垂直同步
- 背景贴图
- 按质量着色 — 红色=重，蓝/绿=轻
- 运动拖尾（保留最近 30 帧位置，渐隐透明度）
- 碰撞高亮闪烁（黄色描边）
- 每个物体中心绘制速度向量

**联机**
- 多个客户端可同时连接同一服务器
- 每帧广播完整世界状态

---

## 操作说明

| 操作 | 效果 |
|---|---|
| 左键按住物体拖动后松开 | 向拖动方向施加冲量 |
| `R` 键 | 在鼠标位置生成矩形 |
| `C` 键 | 在鼠标位置生成圆形 |
| `Escape` / 关闭窗口 | 退出 |

---

## 架构说明

```
服务器  (TCP 0.0.0.0:8080)
  ├─ 物理模拟循环 @ 60 Hz
  ├─ 每帧向所有客户端广播 WorldState JSON
  └─ 接收客户端发来的 ClientMessage JSON

客户端  (SDL2 1200×800)
  ├─ 启动时提示输入服务器 IP:端口（默认 127.0.0.1:8080）
  ├─ 后台线程接收 WorldState
  └─ 主线程渲染画面 + 发送用户输入
```

网络协议：基于原始 TCP 的换行符分隔 JSON。

---

## 编译

**前置条件：** [Rust (stable)](https://rustup.rs) — SDL2 已为 Windows x64 打包在仓库中。

```powershell
cargo build --release
```

产物输出到 `target/release/`。

---

## 运行

```powershell
# 1. 启动服务器
.\target\release\server.exe

# 2. 启动客户端（会提示输入 IP:端口，默认 127.0.0.1:8080）
.\target\release\client.exe
```

也可以直接双击 `start.bat` 快速启动（开发用）。

---

## 发布包结构

分发时将以下文件放在同一目录：

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

所有 DLL 均已包含在仓库中。

---

## 依赖

| Crate | 用途 |
|---|---|
| `sdl2` 0.35 (`image` feature) | 渲染、输入、窗口管理 |
| `serde` / `serde_json` | JSON 网络协议 |

---

## 许可证

MIT
