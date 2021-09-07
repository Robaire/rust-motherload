use std::collections::HashMap;

extern crate gl;
use gl::types::{GLint, GLuint, GLvoid};

extern crate gl_util;
extern crate image;

/// Stores information on every tile in the world
pub struct Map {
    tiles: Vec<Vec<Tile>>,
    size: (u32, u32),
    _tile_textures: HashMap<TileType, GLuint>,
}

impl Map {
    /// Creates a new randomly generated map
    pub fn generate(width: u32, height: u32) -> Map {
        // Get textures for all the tiles
        let texture_map = load_tile_textures();

        // Create the world grid vertex buffer
        let mut world_vertices: Vec<f32> = Vec::new();

        /*
         * Create three layers of air across the entire width of the map,
         * then populate the remaining tiles with regolith
         */

        let mut tiles: Vec<Vec<Tile>> = Vec::new();

        assert!(height > 3);

        // Add the air tiles
        for y in 0..3 {
            let mut row: Vec<Tile> = Vec::new();
            for x in 0..width {
                row.push(Tile {
                    position: (x, y),
                    texture_id: texture_map[&TileType::Air],
                    tile_type: TileType::Air,
                });
            }
            tiles.push(row);
        }

        // Add the regolith tiles
        for y in 3..height {
            let mut row: Vec<Tile> = Vec::new();
            for x in 0..width {
                row.push(Tile {
                    position: (x, y),
                    texture_id: texture_map[&TileType::Regolith],
                    tile_type: TileType::Regolith,
                });
            }
            tiles.push(row);
        }

        Map {
            tiles: tiles,
            size: (width, height),
            _tile_textures: texture_map,
        }
    }

    /// Prints out the world state as text for debugging
    pub fn print(&self) {
        // Iterate over every row in the world
        for y in 0..self.size.1 {
            // Iterate over every element in the row
            for x in 0..self.size.0 {
                // Get the tile type at this location
                match self.tiles[y as usize][x as usize].tile_type {
                    TileType::Air => print!("."),
                    TileType::Regolith => print!("#"),
                    TileType::Ore => print!("O"),
                }
            }

            print!("\n");
        }

        print!("\n");
    }
}

/// Represents a single tile in the world
struct Tile {
    tile_type: TileType,
    // Texutre IDs are stored per tile because tiles of the same type may not
    // have the same texture (for example having special textures for regolith
    // tiles that border air tiles)
    texture_id: GLuint,
    position: (u32, u32),
}

/// Every type of tile that can exist in the world
#[derive(Eq, PartialEq, Hash)]
enum TileType {
    Air,
    Regolith,
    Ore,
}

/// Loads textures for map tiles into the GPU
fn load_tile_textures() -> HashMap<TileType, GLuint> {
    let load_image = |file: &str, idx: i32| {
        let img = image::open(file)
            .expect("Image load failed")
            .flipv()
            .into_rgba8();

        unsafe {
            gl::TexSubImage3D(
                gl::TEXTURE_2D_ARRAY,
                0,
                0,
                0,
                idx,
                16,
                16,
                1,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_ptr() as *const GLvoid,
            );
        }
    };

    let mut id = 0;
    unsafe {
        // Create and bind a texture
        gl::GenTextures(1, &mut id);
        gl::BindTexture(gl::TEXTURE_2D_ARRAY, id);

        // Set Texture Parameters
        gl::TexParameteri(
            gl::TEXTURE_2D_ARRAY,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR as GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D_ARRAY,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as GLint,
        );

        // Allocate memory for the images
        gl::TexStorage3D(gl::TEXTURE_2D_ARRAY, 1, gl::RGBA8, 16, 16, 3);

        // Upload pixel data (NOTE: Does this actually work?)
        load_image("./src/textures/air.png", 0);
        load_image("./src/textures/regolith.png", 1);
        load_image("./src/textures/ore.png", 2);
    }

    // For each type of tile we need to load the appropriate texture
    // TODO: These should be inserted when load_image is called to avoid introducing bugs
    let mut tile_textures: HashMap<TileType, GLuint> = HashMap::new();

    tile_textures.insert(TileType::Air, 0);

    tile_textures.insert(TileType::Regolith, 1);

    tile_textures.insert(TileType::Ore, 2);

    return tile_textures;
}
