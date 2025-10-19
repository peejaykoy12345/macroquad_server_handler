struct Tile{
    x:i32,
    y:i32,
    tile_texture:i32,
    entity_ids:Vec<String>,
}

pub struct Tiles{
    tiles:Vec<Vec<Tile>>,
    tile_size:i32
}

impl Tiles{
    pub fn new(x_max:i32,y_max:i32,tile_size:i32)->Tiles{
        let mut tiles:Vec<Vec<Tile>>=Vec::new();
        for y in 0..y_max{
            let mut row:Vec<Tile>=Vec::new();
            for x in 0..x_max{
                row.push(Tile{x:x*tile_size,y:y*tile_size,tile_texture:1,entity_ids:Vec::new()});
            }
            tiles.push(row);
        }
        Tiles{tiles,tile_size}
    }

    pub fn world_to_tile_index(&self, x: f32, y: f32) -> (usize, usize) {
        let tile_x:usize=(x/self.tile_size as f32).floor()as usize;
        let tile_y:usize=(y/self.tile_size as f32).floor()as usize;
        (tile_x,tile_y)
    }

    pub fn add_entity(&mut self, entity_id: String, x: f32, y: f32) {
        let (tile_x, tile_y) = self.world_to_tile_index(x, y);
        self.tiles[tile_y][tile_x].entity_ids.push(entity_id);
    }

    pub fn remove_entity(&mut self, entity_id: &str, tile_x: usize, tile_y: usize) {
        self.tiles[tile_y][tile_x].entity_ids.retain(|id| id != entity_id);
    }

    pub fn move_entity(&mut self, entity_id: &str, old_x: f32, old_y: f32, new_x: f32, new_y: f32) {
        let (old_tile_x, old_tile_y) = self.world_to_tile_index(old_x, old_y);
        let (new_tile_x, new_tile_y) = self.world_to_tile_index(new_x, new_y);

        if old_tile_x != new_tile_x || old_tile_y != new_tile_y {
            self.remove_entity(entity_id, old_tile_x, old_tile_y);
            self.tiles[new_tile_y][new_tile_x].entity_ids.push(entity_id.to_string());
        }
    }

    pub fn get_entities_in_radius(&self, x: f32, y: f32, radius: isize) -> Vec<String> {
        let (tile_x, tile_y) = self.world_to_tile_index(x, y);
        let mut entity_ids: Vec<String> = Vec::new();

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let ny: isize = tile_y as isize + dy;
                let nx: isize = tile_x as isize + dx;

                if ny >= 0 && nx >= 0
                    && (ny as usize) < self.tiles.len()
                    && (nx as usize) < self.tiles[0].len()
                {
                    entity_ids.extend(self.tiles[ny as usize][nx as usize].entity_ids.clone());
                }
            }
        }
        //println!("COLLISION: {}", crate::utils::format_vector(&entity_ids));
        entity_ids
    }
}