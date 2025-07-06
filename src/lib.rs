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

mod galaga;
mod components;
mod player;
mod npcs;

use galaga::Galaga;
// use server::{GameServer, ServerEventHandler};

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
        let assets = &mut ctx.assets;
        ctx.theme.brand.illustrations.insert(assets, "spaceship");
        ctx.theme.brand.illustrations.insert(assets, "b2");
        ctx.theme.brand.illustrations.insert(assets, "tiki_fly");
        ctx.theme.brand.illustrations.insert(assets, "northrop");
        ctx.theme.brand.illustrations.insert(assets, "bullet_downward");
        ctx.theme.brand.illustrations.insert(assets, "bullet_blue");
        ctx.theme.brand.illustrations.insert(assets, "explosion");

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
            Games::Galaga => Box::new(Galaga::new(ctx))
        }
    }
}