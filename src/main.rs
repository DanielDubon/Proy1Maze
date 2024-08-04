use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use nalgebra_glm::{Vec2};
use std::f32::consts::PI;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::{Instant};
use rusttype::Scale;

mod framebuffer;
use framebuffer::{Framebuffer};
mod maze;
use maze::load_maze;
mod player;
use player::{Player, process_events};

mod caster;
use caster::{cast_ray};

mod texture;
use texture::Texture;

static WALL1: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture:: new("assets/black_wall.png")));
static WALL2: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture:: new("assets/black_wall2.jpg")));

fn cell_to_texture_color(cell: char, tx: u32, ty: u32) -> u32 {
    let default_color = 0x0000000;

    match cell {
        '+' => WALL2.get_pixel_color(tx, ty),
        '-' => WALL1.get_pixel_color(tx, ty),
        '|' => WALL2.get_pixel_color(tx, ty),
        'g' => WALL1.get_pixel_color(tx, ty),
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

fn render3d(framebuffer: &mut Framebuffer, player: &Player) {
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
        let stake_height = (framebuffer.height as f32 / distance) * 70.0;
        
        let stake_top = (hh - (stake_height / 2.0)) as usize;
        let stake_bottom = (hh + (stake_height / 2.0)) as usize;

        

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

fn main() {
    let window_width = 1200;
    let window_height = 720;

    let framebuffer_width = 1200;
    let framebuffer_height = 720;
  
    let frame_duration = Duration::from_secs_f64(0.015);

   

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

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

    while window.is_open() {
        let frame_start_time = Instant::now();
        // listen to inputs
        if window.is_key_down(Key::Escape) {
            break;
        }
        if window.is_key_down(Key::M){
            mode = if mode == "2D" {"3D"} else {"2D"} 
        }


        process_events(&window, &mut player, &maze, block_size); 

        framebuffer.clear();
        if mode == "2D"{
            render2d(&mut framebuffer, &player);
        }else{
            render3d(&mut framebuffer, &player);
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
