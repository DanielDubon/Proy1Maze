use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use nalgebra_glm::{Vec2, distance};
use std::f32::consts::PI;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::{Instant};
use rusttype::{Scale, Font};

mod framebuffer;
use framebuffer::{Framebuffer};
mod maze;
use maze::load_maze;
mod player;
use player::{Player, process_events};
mod audio;
use audio::AudioPlayer;

mod caster;
use caster::{cast_ray};

mod texture;
use texture::Texture;

static WALL1: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture:: new("assets/black_wall.png")));
static WALL2: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture:: new("assets/black_wall2.jpg")));
static ENEMY: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture:: new("assets/fantasma.png")));

fn cell_to_texture_color(cell: char, tx: u32, ty: u32) -> u32 {
    let default_color = 0x0000000;

    match cell {
        '+' => WALL2.get_pixel_color(tx, ty),
        '-' => WALL1.get_pixel_color(tx, ty),
        '|' => WALL2.get_pixel_color(tx, ty),
        'g' => 0x008000,
        _ => default_color,
    }
}


fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, block_size:usize, cell: char) {

    for x in xo..xo + block_size{
        for y in yo..yo + block_size{
            if cell != ' '{
            framebuffer.set_current_color(0x0000000);
            framebuffer.point(x,y);
            }
        }
    }


}

fn render3d(framebuffer: &mut Framebuffer, player: &Player, z_buffer: &mut [f32] ) {
    let maze = load_maze("./maze.txt");
    let block_size = 100;


    for i in 0..framebuffer.width {
        for j in 0..(framebuffer.height / 2){
        framebuffer.set_current_color(0x383838);
        framebuffer.point(i, j)  
    }

        for j in (framebuffer.height / 2)..framebuffer.height{
        framebuffer.set_current_color(0x717171);
        framebuffer.point(i, j)  
    }
}




    let hh = framebuffer.height as f32 / 2.0;

    let num_rays = framebuffer.width;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let Intersect = cast_ray(framebuffer, &maze, player, a, block_size, false);
        
        let distance = Intersect.distance * (a - player.a).cos();
        let mut stake_height = (framebuffer.height as f32 / distance) * 70.0;
        if stake_height > framebuffer.height as f32 {
             stake_height = framebuffer.height as f32;
            }
        let stake_top = (hh - (stake_height / 2.0)) as usize;
        let stake_bottom = (hh + (stake_height / 2.0)) as usize;

        z_buffer[i] = distance;

        for y in stake_top..stake_bottom {
            let ty = (y as f32 - stake_top as f32) / (stake_bottom as f32 - stake_top as f32) * 128.0;
            let tx = Intersect.tx;
            let color = cell_to_texture_color(Intersect.impact, tx as u32, ty as u32);
            framebuffer.set_current_color(color);
            framebuffer.point(i, y)
        }
    }
     

}

fn render2d(framebuffer: &mut Framebuffer, player: &Player) {
    let maze = load_maze("./maze.txt");
    let block_size = 100;

    for row in 0..maze.len(){
        for col in 0..maze[row].len(){
            draw_cell(framebuffer, col * block_size, row * block_size,block_size, maze[row][col]);

        }
    }
    framebuffer.set_current_color(0xFFFFFF);
    framebuffer.point(player.pos.x as usize, player.pos.y as usize);


    let num_rays = 150;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        cast_ray(framebuffer, &maze, player, a, block_size, true);   
    }    
}

fn render_minimap(framebuffer: &mut Framebuffer, maze: &[Vec<char>], block_size: usize, player: &Player) {
    let minimap_size = 200; // Size of the minimap
    let minimap_x = 10; // X position of the minimap
    let minimap_y = 10; // Y position of the minimap

    // Draw minimap background
    for x in minimap_x..minimap_x + minimap_size+70 {
        for y in minimap_y..minimap_y + minimap_size {
            framebuffer.set_current_color(0x222222); // Dark background for the minimap
            framebuffer.point(x, y);
        }
    }

    // Draw maze on the minimap
    let scale = minimap_size as f32 / (maze.len() as f32 * block_size as f32);
    for row in 0..maze.len() {
        for col in 0..maze[row].len() {
            let cell_x = (col as f32 * block_size as f32 * scale) as usize;
            let cell_y = (row as f32 * block_size as f32 * scale) as usize;
            let mini_block_size = (block_size as f32 * scale) as usize;
            draw_cell(framebuffer, minimap_x + cell_x, minimap_y + cell_y, mini_block_size, maze[row][col]);
        }
    }

    // Draw player position on the minimap
    framebuffer.set_current_color(0xFF0000); // Red color for the player
    let player_x = (player.pos.x as f32 * scale) as usize;
    let player_y = (player.pos.y as f32 * scale) as usize;
    framebuffer.point(minimap_x + player_x, minimap_y + player_y);
}

fn render_enemy(framebuffer: &mut Framebuffer, player: &Player, pos: &Vec2, z_buffer: &mut [f32]) {
    // player_a
    let sprite_a = (pos.y - player.pos.y).atan2(pos.x - player.pos.x);
    // let sprite_a = - player.a;
    //
    if sprite_a < 0.0 {
      return;
    }
  
    let sprite_d = ((player.pos.x - pos.x).powi(2) + (player.pos.y - pos.y).powi(2)).sqrt();
    // let sprite_d = distance(player.pos, pos);
  
    if sprite_d < 10.0 {
      return;
    }
  
    let screen_height = framebuffer.height as f32;
    let screen_width = framebuffer.width as f32;
  
    let sprite_size = (screen_height / sprite_d) * 100.0;
    let start_x = (sprite_a - player.a) * (screen_height / player.fov) + (screen_width / 2.0) - (sprite_size / 2.0);
    let start_y = (screen_height / 2.0) - (sprite_size / 2.0);
  
    let end_x = ((start_x + sprite_size) as usize).min(framebuffer.width);
    let end_y = ((start_y + sprite_size) as usize).min(framebuffer.height);
    let start_x = start_x.max(0.0) as usize;
    let start_y = start_y.max(0.0) as usize;
  
    if end_x <= 0 {
      return;
    }
  
    if start_x < framebuffer.width && sprite_d < z_buffer[start_x] {
      for x in start_x..(end_x - 1) {
        for y in start_y..(end_y - 1) {
          let tx = ((x - start_x) * 128 / sprite_size as usize) as u32;
          let ty = ((y - start_y) * 128 / sprite_size as usize) as u32;
          let color = ENEMY.get_pixel_color(tx, ty);
          if color != 0xffffff { 
            framebuffer.set_current_color(color);
            framebuffer.point(x, y);
          }
          z_buffer[x] = sprite_d;
        }
      }
    }
  }
  
  fn render_enemies(framebuffer: &mut Framebuffer, player: &Player, z_buffer: &mut [f32]) {
    let enemies = vec![
      Vec2::new(250.0, 250.0),
      Vec2::new(150.0, 450.0),
    ];
  
    for enemy in &enemies {
      render_enemy(framebuffer, &player, enemy, z_buffer);
    }
  }




fn main() {

    
   
    

    
   let audio_player = AudioPlayer::new("assets/music.mp3").expect("Failed to initialize AudioPlayer");
   audio_player.play();

    
    let steps_player = AudioPlayer::new("assets/steps.mp3").expect("Failed to initialize AudioPlayer");
    
    
    

    let window_width = 1200;
    let window_height = 720;

    let framebuffer_width = 1200;
    let framebuffer_height = 720;
  
    let frame_duration = Duration::from_secs_f64(0.015);

   

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);


    let mut welcome_window = Window::new(
        "Bienvenido - Presiona Enter para jugar",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    let mut welcome_buffer = vec![0; window_width * window_height];

    while welcome_window.is_open() && !welcome_window.is_key_down(Key::Enter) {
        let frame_start_time = Instant::now();
        
        // Dibujar un color de fondo
        for i in 0..welcome_buffer.len() {
            welcome_buffer[i] = 0x000000; // Negro
        }
    
        // Dibujar texto en el buffer de bienvenida
        let scale = Scale::uniform(32.0);
        let text = "Bienvenido, presiona Enter para jugar";
        framebuffer.clear();
        framebuffer.drawtext(&text, 10, 10, scale, 0xFFFFFF); // Asegurarse que el color es 0xFFFFFF para blanco

        
    
        // Actualizar el contenido de `welcome_buffer`
        welcome_window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).unwrap();
    
        // Dormir un poco para evitar consumir demasiada CPU
        let frame_end_time = Instant::now();
        let frame_duration_actual = frame_end_time.duration_since(frame_start_time);
        if frame_duration_actual < frame_duration {
            let sleep_duration = frame_duration - frame_duration_actual;
            if sleep_duration > Duration::from_millis(0) {
                std::thread::sleep(sleep_duration);
            }
        }
    }
    

    // Cerrar la ventana de bienvenida y proceder a la ventana principal
    drop(welcome_window);

    let mut window = Window::new(
        "Rust Graphics - Maze Example",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // move the window around
    window.set_position(100, 100);
    window.update();

    framebuffer.set_background_color(0x333355);

    let mut player = Player {
    pos: Vec2::new(150.0, 150.0),
    a: PI/3.0,
    fov: PI/3.0,
    velocity: Vec2::new(0.0, 0.0),
    previous_mouse_pos: Vec2::new(0.0, 0.0),

    };

    let mut mode = "3D";
    // initialize values

    let maze = load_maze("./maze.txt");
    let block_size = 100;

    let mut last_time = Instant::now();
    let mut frame_count = 0;
    let mut fps_text = String::new();


    let mut goal_position = Vec2::new(0.0, 0.0);
    for (row_idx, row) in maze.iter().enumerate() {
        for (col_idx, &cell) in row.iter().enumerate() {
            if cell == 'g' {
                goal_position = Vec2::new(col_idx as f32 * block_size as f32, row_idx as f32 * block_size as f32);
                break;
            }
        }
    }

    while window.is_open() {
        let frame_start_time = Instant::now();
        // listen to inputs
        if window.is_key_down(Key::Escape) {
            break;
        }
        if window.is_key_down(Key::M){
            mode = if mode == "2D" {"3D"} else {"2D"} 
        }
        if (player.pos - goal_position).norm() < block_size as f32 {
            println!("Has llegado a la meta. ¡Juego terminado!");



            let mut success_window = Window::new(
                "Éxito",
                window_width,
                window_height,
                WindowOptions::default(),
            ).unwrap();

            let mut success_buffer = vec![0; (window_width) * (window_height)];


            while success_window.is_open() {
                // Dibujar fondo negro

                if success_window.is_key_down(Key::Escape) {
                    break;
                }

                for i in 0..success_buffer.len() {
                    success_buffer[i] = 0x000000; // Negro
                }

                
    
                // Dibujar texto de éxito
                let scale = Scale::uniform(32.0);
                let text = "¡PASASTE! ¡FELICIDADES! Presiona ESC para salir";
                framebuffer.clear();
                framebuffer.drawtext(&text, 10, 10, scale, 0xFFFFFF); // Asegurarse que el color es 0xFFFFFF para blanco
    
                // Actualizar la ventana de éxito con el contenido del framebuffer
                success_window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).unwrap();
            }
            break;

        
            
        }
        
        process_events(&window, &mut player, &maze, block_size, &steps_player); 

        framebuffer.clear();
        if mode == "2D"{
            render2d(&mut framebuffer, &player);
        }else{
            let mut z_buffer = vec![f32::INFINITY; framebuffer.width];
            render3d(&mut framebuffer, &player, &mut z_buffer);
            render_enemies(&mut framebuffer, &player, &mut z_buffer);
        }
        render_minimap(&mut framebuffer, &maze, block_size, &player);

        frame_count += 1;
        let current_time = Instant::now();
        let elapsed = current_time.duration_since(last_time);


        if elapsed >= Duration::from_secs(1) {
            let fps = frame_count as f64 / elapsed.as_secs_f64();
            fps_text = format!("FPS: {:.0}", fps);
            last_time = current_time;
            frame_count = 0;
        }

        framebuffer.drawtext(&fps_text, 10, 10, Scale::uniform(32.0), 0xFFFFFF);

        // Update the window with the framebuffer contents
window
.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
.unwrap();

let frame_end_time = Instant::now();
let frame_duration_actual = frame_end_time.duration_since(frame_start_time);
if frame_duration_actual < frame_duration {
    let sleep_duration = frame_duration - frame_duration_actual;
    if sleep_duration > Duration::from_millis(0) {
        std::thread::sleep(sleep_duration);

}
    }
}
}
