mod physics;

use physics::{ClientMessage, RigidBody, Vec2, WorldState, Shape};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::net::TcpStream;
use std::io::{BufReader, BufRead, Write};

fn main() {
    use std::io::{self, Write};
    println!("启动物理客户端...");

    print!("请输入服务器IP (默认127.0.0.1): ");
    io::stdout().flush().unwrap();
    let mut ip = String::new();
    io::stdin().read_line(&mut ip).unwrap();
    let ip = ip.trim();
    let ip = if ip.is_empty() { "127.0.0.1" } else { ip };

    print!("请输入端口 (默认8080): ");
    io::stdout().flush().unwrap();
    let mut port = String::new();
    io::stdin().read_line(&mut port).unwrap();
    let port = port.trim();
    let port = if port.is_empty() { "8080" } else { port };

    let addr = format!("{}:{}", ip, port);
    println!("连接到服务器: {}", addr);

    let stream = match TcpStream::connect(&addr) {
        Ok(s) => {
            println!("连接服务器成功");
            s
        }
        Err(e) => {
            println!("连接服务器失败: {}", e);
            return;
        }
    };

    let world_state = Arc::new(Mutex::new(WorldState { bodies: Vec::new() }));
    let writer = Arc::new(Mutex::new(stream.try_clone().unwrap()));

    let network_world = world_state.clone();
    thread::spawn(move || {
        network_loop(stream, network_world);
    });

    render_loop(world_state, writer);
}

fn network_loop(stream: TcpStream, world_state: Arc<Mutex<WorldState>>) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                println!("服务器断开连接");
                break;
            }
            Ok(_) => {
                if !line.trim().is_empty() {
                    if let Ok(state) = serde_json::from_str::<WorldState>(&line.trim()) {
                        let mut ws = world_state.lock().unwrap();
                        *ws = state;
                        println!("收到新世界状态，物体数量: {}", ws.bodies.len());
                        for b in &ws.bodies {
                            println!("ID: {}, 位置: {:?}, 形状: {:?}", b.id, b.position, b.shape);
                        }
                    } else {
                        println!("收到无法解析的世界状态: {}", line.trim());
                    }
                }
            }
            Err(_) => {
                println!("网络读取错误");
                break;
            }
        }
    }
}

fn render_loop(
    world_state: Arc<Mutex<WorldState>>,
    writer: Arc<Mutex<TcpStream>>,
) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("简单物理沙盒 - 按R添加矩形", 1200, 800)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .unwrap();
        
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut dragging = false;
    let mut drag_body: Option<u32> = None;
    let mut drag_start = Vec2::zero();
    let mut add_rectangle_requested = false;
    let mut add_circle_requested = false;

    let target_fps = 60;
    let frame_duration = Duration::from_nanos(1_000_000_000 / target_fps);

    'running: loop {
        let frame_start = Instant::now();

        // 先收集所有事件，避免在事件循环中访问 event_pump 的其他方法
        let mut events = Vec::new();
        for event in event_pump.poll_iter() {
            events.push(event);
        }

        // 处理收集的事件
        for event in events {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    add_rectangle_requested = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => {
                    add_circle_requested = true;
                }
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    let mouse_pos = Vec2::new(x as f32, y as f32);
                    let ws = world_state.lock().unwrap();
                    
                    for body in &ws.bodies {
                        let delta = mouse_pos - body.position;
                        let is_clicked = match body.shape {
                            Shape::Circle { radius } => delta.length() <= radius,
                            Shape::Rectangle { width, height } => {
                                // 简化的矩形点击检测（不考虑旋转）
                                delta.x.abs() <= width / 2.0 && delta.y.abs() <= height / 2.0
                            }
                        };
                        
                        if is_clicked {
                            dragging = true;
                            drag_body = Some(body.id);
                            drag_start = mouse_pos;
                            break;
                        }
                    }
                }
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    if dragging {
                        if let Some(body_id) = drag_body {
                            let mouse_pos = Vec2::new(x as f32, y as f32);
                            let impulse = (mouse_pos - drag_start) * 5.0;
                            
                            let msg = ClientMessage::ApplyImpulse {
                                body_id,
                                impulse,
                            };
                            
                            let json = serde_json::to_string(&msg).unwrap();
                            let msg_str = format!("{}\n", json);
                            
                            if let Ok(mut w) = writer.lock() {
                                let _ = w.write_all(msg_str.as_bytes());
                                let _ = w.flush();
                            }
                        }
                        dragging = false;
                        drag_body = None;
                    }
                }
                _ => {}
            }
        }

        // 处理添加矩形的请求（在事件循环外获取鼠标状态）
        if add_rectangle_requested {
            let mouse_state = event_pump.mouse_state();
            let mouse_pos = Vec2::new(mouse_state.x() as f32, mouse_state.y() as f32);
            let msg = ClientMessage::AddRectangle {
                position: mouse_pos,
                width: 60.0,
                height: 40.0,
                mass: 1.0,
            };
            let json = serde_json::to_string(&msg).unwrap();
            let msg_str = format!("{}\n", json);
            println!("发送添加矩形请求: {}", msg_str);
            if let Ok(mut w) = writer.lock() {
                let _ = w.write_all(msg_str.as_bytes());
                let _ = w.flush();
            }
            add_rectangle_requested = false;
        }
        if add_circle_requested {
            let mouse_state = event_pump.mouse_state();
            let mouse_pos = Vec2::new(mouse_state.x() as f32, mouse_state.y() as f32);
            let msg = ClientMessage::AddCircle {
                position: mouse_pos,
                radius: 30.0,
                mass: 1.0,
            };
            let json = serde_json::to_string(&msg).unwrap();
            let msg_str = format!("{}\n", json);
            println!("发送添加圆请求: {}", msg_str);
            if let Ok(mut w) = writer.lock() {
                let _ = w.write_all(msg_str.as_bytes());
                let _ = w.flush();
            }
            add_circle_requested = false;
        }

        canvas.set_draw_color(Color::RGB(20, 20, 40));
        canvas.clear();

        let bodies = {
            let ws = world_state.lock().unwrap();
            ws.bodies.clone()
        };
        
        for body in &bodies {
            draw_body(&mut canvas, body);
        }

        // 绘制拖拽线（在事件循环外获取鼠标状态）
        if dragging {
            let mouse_state = event_pump.mouse_state();
            let mouse_pos = Vec2::new(mouse_state.x() as f32, mouse_state.y() as f32);
            
            canvas.set_draw_color(Color::RGB(255, 255, 100));
            canvas.draw_line(
                (drag_start.x as i32, drag_start.y as i32),
                (mouse_pos.x as i32, mouse_pos.y as i32)
            ).unwrap();
        }

        canvas.present();
        
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }
}

fn draw_body(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, body: &RigidBody) {
    match body.shape {
        Shape::Circle { radius } => {
            draw_circle_fast(canvas, body.position, radius, body.mass);
        }
        Shape::Rectangle { width, height } => {
            draw_rectangle_rotated(canvas, body.position, width, height, body.mass, body.angle);
        }
    }
    
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    let velocity_end = body.position + body.velocity * 0.1;
    canvas.draw_line(
        (body.position.x as i32, body.position.y as i32),
        (velocity_end.x as i32, velocity_end.y as i32)
    ).unwrap();
}

fn draw_circle_fast(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, position: Vec2, radius: f32, mass: f32) {
    let cx = position.x as i32;
    let cy = position.y as i32;
    let r = radius as i32;
    
    let color = if mass > 1.5 {
        Color::RGB(250, 100, 100)
    } else {
        Color::RGB(100, 150, 250)
    };
    
    canvas.set_draw_color(color);
    
    let mut x = 0;
    let mut y = r;
    let mut d = 3 - 2 * r;
    
    while y >= x {
        for &(dx, dy) in &[(x, y), (-x, y), (x, -y), (-x, -y), 
                           (y, x), (-y, x), (y, -x), (-y, -x)] {
            canvas.draw_point((cx + dx, cy + dy)).ok();
        }
        
        if d < 0 {
            d = d + 4 * x + 6;
        } else {
            d = d + 4 * (x - y) + 10;
            y -= 1;
        }
        x += 1;
    }
}

fn draw_rectangle_rotated(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    position: Vec2,
    width: f32,
    height: f32,
    mass: f32,
    angle: f32,
) {
    let color = if mass > 1.5 {
        Color::RGB(250, 100, 100)
    } else {
        Color::RGB(100, 250, 100)
    };
    canvas.set_draw_color(color);

    // 计算四个角点
    let hw = width / 2.0;
    let hh = height / 2.0;
    let corners = [
        Vec2 { x: -hw, y: -hh },
        Vec2 { x: hw, y: -hh },
        Vec2 { x: hw, y: hh },
        Vec2 { x: -hw, y: hh },
    ];
    let rotated: Vec<(i32, i32)> = corners
        .iter()
        .map(|c| {
            let x = c.x * angle.cos() - c.y * angle.sin();
            let y = c.x * angle.sin() + c.y * angle.cos();
            ((position.x + x) as i32, (position.y + y) as i32)
        })
        .collect();

    // 填充多边形（近似，先画边）
    for i in 0..4 {
        let (x1, y1) = rotated[i];
        let (x2, y2) = rotated[(i + 1) % 4];
        canvas.draw_line((x1, y1), (x2, y2)).ok();
    }

    // 画白色边框
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for i in 0..4 {
        let (x1, y1) = rotated[i];
        let (x2, y2) = rotated[(i + 1) % 4];
        canvas.draw_line((x1, y1), (x2, y2)).ok();
    }
}