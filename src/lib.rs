mod game_collection;

use pelican_ui::{Context, Plugins, Plugin, maverick_start, start, Application, PelicanEngine, MaverickOS, HardwareContext, runtime::Services};
use pelican_ui::drawable::Drawable;
use pelican_ui_std::{AvatarIconStyle, AvatarContent, Interface, NavigateEvent, AppPage};
use pelican_ui::runtime::{Service, ServiceList};
use std::any::TypeId;
use std::pin::Pin;
use std::future::Future;
use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey};
use std::collections::BTreeMap;
use std::os::unix::raw::mode_t;
use std::ptr::addr_of_mut;
use image::{load_from_memory, RgbaImage};
use pelican_ui::include_assets;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use crate::game_collection::galaga_game::galaga::Galaga;
use crate::game_collection::galaga_game::player::Player;
use crate::game_collection::galaga_game::server::{ArduinoServer, GameAction};

pub struct MyApp;

impl Services for MyApp {
    fn services() -> ServiceList {
        ServiceList::default()
    }
}

impl Plugins for MyApp {
    fn plugins(ctx: &mut Context) -> Vec<Box<dyn Plugin>> {
        vec![]
    }
}

impl Application for MyApp {
    async fn new(ctx: &mut Context) -> Box<dyn Drawable> {
        
        ctx.assets.include_assets(include_assets!("./assets"));
        let mut illustrations = ctx.theme.brand.illustrations.clone();
        illustrations.insert(ctx, "spaceship", "spaceship.png");
        illustrations.insert(ctx, "b2", "b2.png");
        illustrations.insert(ctx, "tiki_fly", "tiki_fly.png");
        illustrations.insert(ctx, "northrop", "northrop.png");
        illustrations.insert(ctx, "bullet_downward", "bullet_downward.png");
        illustrations.insert(ctx, "bullet_blue", "bullet_blue.png");
        illustrations.insert(ctx, "explosion", "explosion.png");
        ctx.theme.brand.illustrations = illustrations;

        let game = Games::Galaga.init(ctx);
        Box::new(Interface::new(ctx, game, None))
    }
}

start!(MyApp);

enum Games {
    Galaga
}

impl Games {
    pub fn init(&self, ctx: &mut Context) -> Box<dyn AppPage> {
        match self {
            Games::Galaga => Box::new(Galaga::new(ctx, None))
        }
    }
}