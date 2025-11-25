mod physics;

use physics::{RigidBody, Vec2, WorldState, ClientMessage, Shape};
use std::collections::HashMap;
use std::io::{BufReader, BufRead, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    println!("启动物理服务器...");

    let world_state = Arc::new(Mutex::new(WorldState {
        bodies: vec![
            RigidBody::new_circle(1, Vec2::new(200.0, 300.0), 30.0, 2.0),
            RigidBody::new_circle(2, Vec2::new(400.0, 200.0), 25.0, 1.0),
            RigidBody::new_rectangle(3, Vec2::new(600.0, 400.0), 80.0, 60.0, 3.0),
            RigidBody::new_rectangle(4, Vec2::new(300.0, 500.0), 50.0, 50.0, 0.5),
        ],
    }));

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("服务器监听在 127.0.0.1:8080");

    let clients: Arc<Mutex<HashMap<usize, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut client_counter = 0;

    let simulation_world = world_state.clone();
    let simulation_clients = clients.clone();
    thread::spawn(move || {
        simulation_loop(simulation_world, simulation_clients);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("新的客户端连接");
                client_counter += 1;
                let client_id = client_counter;

                let world = world_state.clone();
                let clients_map = clients.clone();
                let stream_clone = stream.try_clone().unwrap();

                clients.lock().unwrap().insert(client_id, stream);

                thread::spawn(move || {
                    let mut reader = BufReader::new(stream_clone);
                    let mut line = String::new();

                    loop {
                        line.clear();
                        match reader.read_line(&mut line) {
                            Ok(0) => {
                                println!("客户端 {} 断开连接", client_id);
                                break;
                            }
                            Ok(_) => {
                                if !line.trim().is_empty() {
                                    if let Ok(message) = serde_json::from_str::<ClientMessage>(&line.trim()) {
                                        handle_client_message(message, world.clone());
                                    }
                                }
                            }
                            Err(_) => {
                                println!("从客户端 {} 读取错误", client_id);
                                break;
                            }
                        }
                    }

                    clients_map.lock().unwrap().remove(&client_id);
                });
            }
            Err(e) => {
                println!("连接错误: {}", e);
            }
        }
    }
}

fn handle_client_message(message: ClientMessage, world: Arc<Mutex<WorldState>>) {
    println!("收到客户端消息: {:?}", message);
    match message {
        ClientMessage::ApplyImpulse { body_id, impulse } => {
            let mut world = world.lock().unwrap();
            if let Some(body) = world.bodies.iter_mut().find(|b| b.id == body_id) {
                body.velocity = body.velocity + impulse * (1.0 / body.mass);
                println!("对物体 {} 施加冲量: {:?}", body_id, impulse);
            }
        }
        ClientMessage::AddRectangle { position, width, height, mass } => {
            let mut world = world.lock().unwrap();
            let new_id = world.bodies.iter().map(|b| b.id).max().unwrap_or(0) + 1;
            let new_rect = RigidBody::new_rectangle(new_id, position, width, height, mass);
            world.bodies.push(new_rect);
            println!("添加新矩形，ID: {}, 位置: {:?}", new_id, position);
            println!("当前物体列表:");
            for b in &world.bodies {
                println!("ID: {}, 位置: {:?}, 形状: {:?}", b.id, b.position, b.shape);
            }
        }
        ClientMessage::AddCircle { position, radius, mass } => {
            let mut world = world.lock().unwrap();
            let new_id = world.bodies.iter().map(|b| b.id).max().unwrap_or(0) + 1;
            let new_circle = RigidBody::new_circle(new_id, position, radius, mass);
            world.bodies.push(new_circle);
            println!("添加新圆，ID: {}, 位置: {:?}, 半径: {}", new_id, position, radius);
            println!("当前物体列表:");
            for b in &world.bodies {
                println!("ID: {}, 位置: {:?}, 形状: {:?}", b.id, b.position, b.shape);
            }
        }
    }
}

fn simulation_loop(world: Arc<Mutex<WorldState>>, clients: Arc<Mutex<HashMap<usize, TcpStream>>>) {
    let fixed_dt = 1.0 / 60.0;
    let step_duration = Duration::from_secs_f32(fixed_dt);

    loop {
        let step_start = Instant::now();

        {
            let mut world = world.lock().unwrap();
            
            for body in &mut world.bodies {
                // 重力
                body.velocity.y += 98.0 * fixed_dt;
                
                // 更新位置
                body.position = body.position + body.velocity * fixed_dt;
                
                // 更新角度
                body.angle += body.angular_velocity * fixed_dt;
                
                // 边界碰撞检测 - 根据形状类型
                match body.shape {
                    Shape::Circle { radius } => {
                        if body.position.x - radius < 0.0 {
                            body.position.x = radius;
                            body.velocity.x = -body.velocity.x * 0.8;
                        } else if body.position.x + radius > 1200.0 {
                            body.position.x = 1200.0 - radius;
                            body.velocity.x = -body.velocity.x * 0.8;
                        }
                        
                        if body.position.y - radius < 0.0 {
                            body.position.y = radius;
                            body.velocity.y = -body.velocity.y * 0.8;
                        } else if body.position.y + radius > 800.0 {
                            body.position.y = 800.0 - radius;
                            body.velocity.y = -body.velocity.y * 0.8;
                        }
                    }
                    Shape::Rectangle { width, height } => {
                        // 简化的矩形边界碰撞（不考虑旋转）
                        let half_width = width / 2.0;
                        let half_height = height / 2.0;
                        
                        if body.position.x - half_width < 0.0 {
                            body.position.x = half_width;
                            body.velocity.x = -body.velocity.x * 0.8;
                            // 添加一些角速度使碰撞更有趣
                            body.angular_velocity += body.velocity.y * 0.01;
                        } else if body.position.x + half_width > 1200.0 {
                            body.position.x = 1200.0 - half_width;
                            body.velocity.x = -body.velocity.x * 0.8;
                            body.angular_velocity += body.velocity.y * 0.01;
                        }
                        
                        if body.position.y - half_height < 0.0 {
                            body.position.y = half_height;
                            body.velocity.y = -body.velocity.y * 0.8;
                            body.angular_velocity += body.velocity.x * 0.01;
                        } else if body.position.y + half_height > 800.0 {
                            body.position.y = 800.0 - half_height;
                            body.velocity.y = -body.velocity.y * 0.8;
                            body.angular_velocity += body.velocity.x * 0.01;
                        }
                    }
                }
                
                // 阻尼
                body.velocity = body.velocity * 0.995;
                body.angular_velocity *= 0.99; // 角速度阻尼
            }
            
            // 简化的碰撞检测
            let body_count = world.bodies.len();
            for i in 0..body_count {
                for j in i + 1..body_count {
                    let pos_i = world.bodies[i].position;
                    let pos_j = world.bodies[j].position;
                    let vel_i = world.bodies[i].velocity;
                    let vel_j = world.bodies[j].velocity;
                    let mass_i = world.bodies[i].mass;
                    let mass_j = world.bodies[j].mass;
                    let shape_i = world.bodies[i].shape;
                    let shape_j = world.bodies[j].shape;
                    
                    let (min_i, max_i) = get_bounding_box_from_data(pos_i, shape_i);
                    let (min_j, max_j) = get_bounding_box_from_data(pos_j, shape_j);
                    
                    if max_i.x >= min_j.x && min_i.x <= max_j.x &&
                       max_i.y >= min_j.y && min_i.y <= max_j.y {
                        let normal = (pos_i - pos_j).normalize();
                        let overlap = calculate_overlap_from_data(pos_i, shape_i, pos_j, shape_j);
                        
                        if overlap > 0.0 {
                            world.bodies[i].position = world.bodies[i].position + normal * (overlap * 0.5);
                            world.bodies[j].position = world.bodies[j].position - normal * (overlap * 0.5);
                            
                            let relative_velocity = vel_i - vel_j;
                            let velocity_along_normal = relative_velocity.x * normal.x + relative_velocity.y * normal.y;
                            
                            if velocity_along_normal > 0.0 {
                                continue;
                            }
                            
                            let restitution = 0.8;
                            let mut impulse_magnitude = -(1.0 + restitution) * velocity_along_normal;
                            impulse_magnitude /= 1.0 / mass_i + 1.0 / mass_j;
                            
                            let impulse = normal * impulse_magnitude;
                            
                            world.bodies[i].velocity = world.bodies[i].velocity + impulse * (1.0 / mass_i);
                            world.bodies[j].velocity = world.bodies[j].velocity - impulse * (1.0 / mass_j);
                            
                            // 添加一些角速度
                            world.bodies[i].angular_velocity += impulse_magnitude * 0.001;
                            world.bodies[j].angular_velocity -= impulse_magnitude * 0.001;
                        }
                    }
                }
            }
            
            let world_json = serde_json::to_string(&*world).unwrap();
            let message = format!("{}\n", world_json);
            
            let mut clients = clients.lock().unwrap();
            let mut disconnected = Vec::new();
            
            for (&client_id, stream) in clients.iter_mut() {
                if let Err(_) = stream.write_all(message.as_bytes()) {
                    disconnected.push(client_id);
                }
                let _ = stream.flush();
            }
            
            for client_id in disconnected {
                clients.remove(&client_id);
                println!("客户端 {} 断开连接", client_id);
            }
        }

        let elapsed = step_start.elapsed();
        if elapsed < step_duration {
            thread::sleep(step_duration - elapsed);
        }
    }
}

fn get_bounding_box_from_data(position: Vec2, shape: Shape) -> (Vec2, Vec2) {
    match shape {
        Shape::Circle { radius } => {
            let min = Vec2::new(position.x - radius, position.y - radius);
            let max = Vec2::new(position.x + radius, position.y + radius);
            (min, max)
        }
        Shape::Rectangle { width, height } => {
            // 简化的包围盒（不考虑旋转）
            let half_width = width / 2.0;
            let half_height = height / 2.0;
            let min = Vec2::new(position.x - half_width, position.y - half_height);
            let max = Vec2::new(position.x + half_width, position.y + half_height);
            (min, max)
        }
    }
}

fn calculate_overlap_from_data(pos_i: Vec2, shape_i: Shape, pos_j: Vec2, shape_j: Shape) -> f32 {
    let (min_i, max_i) = get_bounding_box_from_data(pos_i, shape_i);
    let (min_j, max_j) = get_bounding_box_from_data(pos_j, shape_j);
    
    let overlap_x = (max_i.x - min_j.x).min(max_j.x - min_i.x);
    let overlap_y = (max_i.y - min_j.y).min(max_j.y - min_i.y);
    
    if overlap_x < 0.0 || overlap_y < 0.0 {
        0.0
    } else {
        overlap_x.min(overlap_y)
    }
}