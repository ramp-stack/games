use std::any::Any;
use std::arch::aarch64::__crc32b;
use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding, Column, Wrap, Row, ButtonSize, ButtonWidth, ButtonStyle, ButtonState, IconButton, NavigateEvent, DataItem};
use pelican_game_engine::{AspectRatio, Sprite, Gameboard, SpriteState, SpriteAction, CollisionEvent};

use crate::player::Player;
use crate::npcs::{Enemy, EnemyPatterns, Bullet, Explosion};

use std::time::Instant;

#[derive(Debug, Default, Clone)]
pub struct GameState {
    pub player: Option<Player>,
    pub enemies: Option<Vec<Enemy>>,
    pub bullets: Vec<Bullet>,
    pub explosions: Vec<Explosion>,
    pub interval: Option<Instant>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            player: None,
            enemies: None,
            bullets: Vec::new(),
            explosions: Vec::new(),
            interval: Some(Instant::now()),
        }
    }
}

#[derive(Debug, Component)]
pub struct Galaga(Column, Header, Gameboard);
impl OnEvent for Galaga {}

impl AppPage for Galaga {
    fn has_nav(&self) -> bool {false}
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
        match index {
            0 => Ok(self),//Ok(Box::new(Settings::new(ctx))),
            _ => Err(self)
        }
    }
}

impl Galaga {
    pub fn new(ctx: &mut Context) -> Self {
        let mut gamestate = GameState::new();
        let mut gameboard = Gameboard::new(ctx, AspectRatio::OneOne, Box::new(Self::on_event));

        let player = Player::new(ctx, &mut gameboard);
        gamestate.player = Some(player);

        ctx.state().set(gamestate);
        let settings = IconButton::navigation(ctx, "settings", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let header = Header::stack(ctx, None, "Galaga", Some(settings));
        Galaga(Column::center(24.0), header, gameboard)
    }


    fn on_event(gameboard: &mut Gameboard, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            let mut player = gamestate.player.clone();

            player.as_mut().map(|p| p.react(ctx, gameboard));
            
            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            gamestate.player = player;

            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            let mut enemies = gamestate.enemies.clone();
            
            if enemies.is_none() {
                let new_enemies = EnemyPatterns::Star.get(ctx, gameboard).into_iter()
                    .map(|(s, id)| Enemy::new(ctx, gameboard, s, id)).collect::<Vec<Enemy>>();
                enemies = Some(new_enemies);
            }

            enemies.as_mut().map(|es| es.iter_mut().for_each(|e| e.react(ctx, gameboard)));

            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            gamestate.enemies = enemies;

            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            let mut bullets = gamestate.bullets.clone();

            bullets.retain_mut(|b| b.react(ctx, gameboard));

            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            gamestate.bullets = bullets;

            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            let mut explosions = gamestate.explosions.clone();

            explosions.as_mut().map(|e| e.react(ctx, gameboard));
            
            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            gamestate.explosions = explosions;

            let mut gamestate = ctx.state().get_mut_or_default::<GameState>().clone();

            let (maxw, maxh) = gameboard.0.size(ctx);
            gameboard.2.iter_mut().enumerate().for_each(|(i, s)| {
                if let Some(location) = gameboard.0.0.get_mut(i+1) {
                    let (x, y) = s.position(ctx);
                    location.0 = Offset::Static(x);
                    location.1 = Offset::Static(y);
                }
                //  TODO: Need to keep everything a percentage of screen size
            });

            ctx.state().set(gamestate);
        } else if let Some(CollisionEvent(a, b)) = event.downcast_ref::<CollisionEvent>() {
            let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
            println!("{:?} collided into {:?}", b, a);
            if a.starts_with("player") && b.starts_with("missile") { // enemy bullet hit player ship
                gamestate.bullets.retain_mut(|bu| bu.id() != *b);
                gameboard.remove_sprite_by_id(b);
            } else if a.starts_with("enemy") && b.starts_with("bullet") { // player bullet hit enemy ship
                gamestate.bullets.retain_mut(|bu| bu.id() != *b);
                gameboard.remove_sprite_by_id(b);

                let enemy = gameboard.get_sprite_by_id(a).unwrap();
                let pos = enemy.position(ctx);

                let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
                gamestate.enemies.as_mut().unwrap().retain_mut(|e| e.id() != *a);
                gameboard.remove_sprite_by_id(a);

                let explosion = Explosion::new(ctx, gameboard, pos.0, pos.1);
                gameboard.explosions.push(explosion);
            }
        } else if let Some(keyboard_event) = event.downcast_ref::<KeyboardEvent>() {
            let gamestate = ctx.state().get_mut_or_default::<GameState>();
            match keyboard_event {
                KeyboardEvent { state: KeyboardState::Pressed, key: Key::Named(NamedKey::ArrowLeft) } => {
                    gamestate.player.as_mut().map(|p| p.set_state(SpriteState::MovingLeft));
                }
                KeyboardEvent { state: KeyboardState::Released, key: Key::Named(NamedKey::ArrowLeft) } => {
                    gamestate.player.as_mut().map(|p| p.set_state(SpriteState::Idle));
                }
                KeyboardEvent { state: KeyboardState::Pressed, key: Key::Named(NamedKey::ArrowRight) } => {
                    gamestate.player.as_mut().map(|p| p.set_state(SpriteState::MovingRight));
                }
                KeyboardEvent { state: KeyboardState::Released, key: Key::Named(NamedKey::ArrowRight) } => {
                    gamestate.player.as_mut().map(|p| p.set_state(SpriteState::Idle));
                }
                KeyboardEvent { state: KeyboardState::Pressed, key: Key::Named(NamedKey::ArrowUp) } => {
                    gamestate.player.as_mut().map(|p| p.action(SpriteAction::Shoot));
                }
                _ => {}
            }
        }
        true
    }
}