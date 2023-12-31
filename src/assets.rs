use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "ship.png")]
    pub ship: Handle<Image>,
    #[asset(path = "boat.png")]
    pub boat: Handle<Image>,

    #[asset(path = "small_ship_1.png")]
    pub small_ship_1: Handle<Image>,
    #[asset(path = "station.png")]
    pub station: Handle<Image>,

    #[asset(path = "cannon_turret.png")]
    pub cannon_turret: Handle<Image>,
    #[asset(path = "rocket_turret.png")]
    pub rocket_turret: Handle<Image>,
    #[asset(path = "medium_rocket_turret.png")]
    pub medium_rocket_turret: Handle<Image>,

    #[asset(path = "water.png")]
    pub water: Handle<Image>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 4, rows = 1))]
    #[asset(path = "cannon.png")]
    pub cannon: Handle<TextureAtlas>,

    #[asset(path = "rocket.png")]
    pub rocket: Handle<Image>,
    #[asset(path = "medium_rocket.png")]
    pub medium_rocket: Handle<Image>,

    #[asset(texture_atlas(tile_size_x = 32.0, tile_size_y = 32.0, columns = 8, rows = 1))]
    #[asset(path = "gfx/explosion.png")]
    pub explosion: Handle<TextureAtlas>,
}
