use std::fs::File;
use std::io::{Write, Error};
use std::io::LineWriter;
use rand::Rng;

fn pack_color(r: u8, g: u8, b: u8) -> u32 {
    let a: u8 = 255;
    (u32::from(a) << 24) + (u32::from(b) << 16) + (u32::from(g) << 8) + u32::from(r)
}

fn unpack_color(color: u32) -> (u8, u8, u8, u8) {
    let r: u8 = ((color >> 0) & 255).try_into().unwrap();
    let g: u8 = ((color >> 8) & 255).try_into().unwrap();
    let b: u8 = ((color >> 16) & 255).try_into().unwrap();
    let a: u8 = ((color >> 24) & 255).try_into().unwrap();

    (r, g, b, a)
}

fn drop_ppm_image(filename: &str, image: &Vec<u32>, w: usize, h: usize) -> Result<(), Error> {
    let file = File::create(filename)?;
    let mut file = LineWriter::new(file);
    writeln!(file, "P6")?;
    writeln!(file, "{} {}", w, h)?;
    writeln!(file, "255")?;
    for i in 0..h*w {
        let (r, g, b, _a): (u8, u8, u8, u8) = unpack_color(image[i]);
        file.write_all(&[r, g, b])?;
    }

    file.flush()?;
    Ok(())
}

fn draw_rectangle(img: &mut Vec<u32>, img_w: usize, img_h: usize, x: usize, y: usize, w: usize, h: usize, color: u32) {
    assert_eq!(img.len(), img_w * img_h);
    for i in 0..w {
        for j in 0..h {
            let cx = x + i;
            let cy = y + j;
            if cx >= img_w || cy >= img_h { continue; }
            img[cx + cy * img_w] = color;
        }
    }
} 

fn main() {
    const WIN_W: usize = 1024;  // image width
    const WIN_H: usize = 512;   // image height
    let mut framebuffer: Vec<u32> = vec![pack_color(255, 255, 255); WIN_W * WIN_H]; // image initiliazed to white

    let player_x = 3.456f32; // player x position
    let player_y = 2.345f32; // player y position
    let mut player_a = 1.523f32; // player's angle of view
    const FOV: f32 = std::f32::consts::PI / 3.; // field of view

    const MAP_W: usize = 16; // map width
    const MAP_H: usize = 16; // map height
    let map_str = 
    "0000222222220000\
     1              0\
     1      11111   0\
     1     0        0\
     0     0  1110000\
     0     3        0\
     0   10000      0\
     0   0   11100  0\
     0   0   0      0\
     0   0   1  00000\
     0       1      0\
     2       1      0\
     0       0      0\
     0 0000000      0\
     0              0\
     0002222222200000"; 
    //let map = map_str.graphemes(true).collect::<Vec<&str>>(); // our game map
    let map: Vec<char> = map_str.chars().collect();
     assert_eq!(MAP_W * MAP_H, map.len());

    /* 
    for j in 0..WIN_H {
        for i in 0..WIN_W {
            let r: u8 = (255 * j / WIN_H).try_into().unwrap();
            let g: u8 = (255 * i / WIN_W).try_into().unwrap();
            let b: u8 = 0;
            framebuffer[i + j * WIN_W] = pack_color(r, g, b);
        }
    } */

    const NCOLORS: usize = 10;
    let mut colors: Vec<u32> = vec![0; NCOLORS];
    for i in 0..NCOLORS {
        let r: u8 = rand::thread_rng().gen_range(0..=255);
        let g: u8 = rand::thread_rng().gen_range(0..=255);
        let b: u8 = rand::thread_rng().gen_range(0..=255);
        colors[i] = pack_color(r, g, b);
    }

    const RECT_W: usize = WIN_W / (MAP_W * 2);
    const RECT_H: usize = WIN_H / MAP_H;

    for frame in 0..360 {
        let filename = format!("./output/{:0>5}.ppm", frame);
        player_a += 2.0 * std::f32::consts::PI / 360.0;

        // framebuffer = vec![pack_color(255, 255, 255); WIN_W * WIN_H]; // clear the screen

        for j in 0..MAP_H {
            for i in 0..MAP_W {
                if map[i + j * MAP_W] == ' ' { continue; } // skip empty spaces
                let rect_x = i * RECT_W;
                let rect_y = j * RECT_H;
                let icolor = (map[i + j * MAP_W] as u32 - '0' as u32) as usize;
                assert!(icolor < NCOLORS);
                draw_rectangle(&mut framebuffer, WIN_W, WIN_H, rect_x, rect_y, RECT_W, RECT_H, colors[icolor]);
            }
        }

        for i in 0..(WIN_W / 2) { // draw visibility cone
            let angle = player_a - FOV / 2.0 + FOV * i as f32 / ((WIN_W / 2) as f32);
    
            let mut t = 0.0;
            while t < 20.0 {
                let cx = player_x + t * angle.cos();
                let cy = player_y + t * angle.sin();
                // if map[cx as usize + cy as usize * MAP_W] != ' ' { break; }
    
                // Note to self: when multiplying a floating point by an integer
                // always convert the integer to floating point before performing
                // the math operation, then cast back into an integer value if
                // that's what you need. Converting the floating point to an
                // integer first will lose a lot of precision
                let pix_x = (cx * RECT_W as f32) as usize;
                let pix_y = (cy * RECT_H as f32) as usize;
                framebuffer[pix_x + pix_y * WIN_W] = pack_color(160, 160, 160); // draws visbility cone on map
    
                if map[cx as usize + cy as usize * MAP_W] != ' ' {
                    // our ray touches a wall, so draw a vertical column
                    let icolor = (map[cx as usize + cy as usize * MAP_W] as u32 - '0' as u32) as usize;
                    assert!(icolor < NCOLORS);
                    let column_height = (WIN_H as f32 / (t * (angle - player_a).cos())) as usize;
                    draw_rectangle(&mut framebuffer, WIN_W, WIN_H, WIN_W / 2 + i, WIN_H / 2 - column_height / 2, 1, column_height, colors[icolor]);
                    break;
                }
    
                t += 0.01;
            } 
        }

        let _ = drop_ppm_image(&filename, &framebuffer, WIN_W, WIN_H);

        framebuffer = vec![pack_color(255, 255, 255); WIN_W * WIN_H]; // clear the screen
    }

    

    
    

    // let _ = drop_ppm_image("./out.ppm", &framebuffer, WIN_W, WIN_H);
}
