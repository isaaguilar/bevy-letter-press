use bevy::ecs::resource::Resource;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]

pub struct ImageAssets {
    #[asset(path = "title.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub title: Handle<Image>,

    #[asset(path = "clock.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub clock: Handle<Image>,

    #[asset(path = "healthbar-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub healthbar: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 101, tile_size_y = 27, columns = 9, rows = 1))]
    pub healthbar_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "sfx-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub sfx: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 18, tile_size_y = 18, columns = 2, rows = 1))]
    pub sfx_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "volume-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub volume: Handle<Image>,

    #[asset(path = "weed1-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub weed1: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 250, tile_size_y = 480, columns = 9, rows = 1))]
    pub weed1_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "weed2-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub weed2: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 400, tile_size_y = 480, columns = 9, rows = 1))]
    pub weed2_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "weed3-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub weed3: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 208, tile_size_y = 480, columns = 9, rows = 1))]
    pub weed3_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "weed4-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub weed4: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 400, tile_size_y = 480, columns = 9, rows = 1))]
    pub weed4_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "weed5-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub weed5: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 400, tile_size_y = 480, columns = 9, rows = 1))]
    pub weed5_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "farm.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub farm: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 18, tile_size_y = 18, columns = 2, rows = 1))]
    pub volume_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "letterbox-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub letterbox: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 9, rows = 1))]
    pub letterbox_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "modal.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub modal: Handle<Image>,

    #[asset(path = "instructions-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub instructions: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 568, tile_size_y = 259, columns = 7, rows = 1))]
    pub instructions_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "rose-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub rose: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 100, tile_size_y = 480, columns = 10, rows = 1))]
    pub rose_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "letters-Sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub letters: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 28, tile_size_y = 28, columns = 26, rows = 1))]
    pub letters_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "circle-transition.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub circle_transition: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 512, tile_size_y = 320, columns = 14, rows = 1))]
    pub circle_transition_layout: Handle<TextureAtlasLayout>,
}

#[derive(AssetCollection, Resource)]

pub struct SoundAssets {
    #[asset(path = "sfx/tropical.ogg")]
    pub music: Handle<AudioSource>,

    #[asset(path = "sfx/win.ogg")]
    pub win: Handle<AudioSource>,

    #[asset(path = "sfx/lose.ogg")]
    pub lose: Handle<AudioSource>,

    #[asset(path = "sfx/error.ogg")]
    pub error: Handle<AudioSource>,

    #[asset(path = "sfx/menu.ogg")]
    pub menu_music: Handle<AudioSource>,

    #[asset(path = "sfx/collect.ogg")]
    pub collect_sfx: Handle<AudioSource>,

    #[asset(path = "sfx/walk1.ogg")]
    pub walk1: Handle<AudioSource>,
    #[asset(path = "sfx/walk2.ogg")]
    pub walk2: Handle<AudioSource>,
    #[asset(path = "sfx/walk3.ogg")]
    pub walk3: Handle<AudioSource>,
    #[asset(path = "sfx/walk4.ogg")]
    pub walk4: Handle<AudioSource>,
    #[asset(path = "sfx/walk5.ogg")]
    pub walk5: Handle<AudioSource>,
    #[asset(path = "sfx/walk6.ogg")]
    pub walk6: Handle<AudioSource>,
    #[asset(path = "sfx/walk7.ogg")]
    pub walk7: Handle<AudioSource>,
    #[asset(path = "sfx/walk8.ogg")]
    pub walk8: Handle<AudioSource>,
    #[asset(path = "sfx/walk9.ogg")]
    pub walk9: Handle<AudioSource>,
    #[asset(path = "sfx/walk10.ogg")]
    pub walk10: Handle<AudioSource>,

    #[asset(path = "sfx/boingjump1.ogg")]
    pub boingjump1: Handle<AudioSource>,
    #[asset(path = "sfx/boingjump2.ogg")]
    pub boingjump2: Handle<AudioSource>,

    #[asset(path = "sfx/impact1.ogg")]
    pub impact1: Handle<AudioSource>,
    #[asset(path = "sfx/impact2.ogg")]
    pub impact2: Handle<AudioSource>,
    #[asset(path = "sfx/impact3.ogg")]
    pub impact3: Handle<AudioSource>,

    #[asset(path = "sfx/jump1.ogg")]
    pub swoosh1: Handle<AudioSource>,
    #[asset(path = "sfx/jump2.ogg")]
    pub swoosh2: Handle<AudioSource>,
    #[asset(path = "sfx/jump2.ogg")]
    pub swoosh3: Handle<AudioSource>,
    #[asset(path = "sfx/jump4.ogg")]
    pub swoosh4: Handle<AudioSource>,

    #[asset(path = "sfx/thud1.ogg")]
    pub thud1: Handle<AudioSource>,
    #[asset(path = "sfx/thud2.ogg")]
    pub thud2: Handle<AudioSource>,
    #[asset(path = "sfx/thud3.ogg")]
    pub thud3: Handle<AudioSource>,
}
