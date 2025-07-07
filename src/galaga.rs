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
use crate::server::GameAction;

use std::time::Instant;
use rand::thread_rng;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

#[derive(Debug, Default, Clone)]
pub struct GameState {
    pub player: Option<Player>,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub explosions: Vec<Explosion>,
    pub interval: Option<Instant>,
    pub action_queue: Option<Arc<Mutex<VecDeque<GameAction>>>>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            player: None,
            enemies: Vec::new(),
            bullets: Vec::new(),
            explosions: Vec::new(),
            interval: Some(Instant::now()),
            action_queue: None,
        }
    }

    pub fn set_action_queue(&mut self, queue: Arc<Mutex<VecDeque<GameAction>>>) {
        self.action_queue = Some(queue);
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
    pub fn new(ctx: &mut Context, action_queue: Arc<Mutex<VecDeque<GameAction>>>) -> Self {
        let mut gamestate = GameState::new();
        gamestate.set_action_queue(action_queue);
        
        let mut gameboard = Gameboard::new(ctx, AspectRatio::OneOne, Box::new(Self::on_event));

        let mut player = Player::new(ctx, &mut gameboard);
        
        player.set_auto_shoot(true);
        player.set_auto_move(false);
        
        player.player_lives_display(ctx, &mut gameboard);
        
        gamestate.player = Some(player);
        
        ctx.state().set(gamestate);
        let settings = IconButton::navigation(ctx, "settings", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let header = Header::stack(ctx, None, "Galaga", Some(settings));
        Galaga(Column::center(24.0), header, gameboard)
    }

    fn on_event(gameboard: &mut Gameboard, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            
            if let Some(ref action_queue) = gamestate.action_queue.clone() {
                if let Ok(mut queue) = action_queue.lock() {
                    while let Some(action) = queue.pop_front() {
                        match action {
                            GameAction::MoveLeft => {
                                if let Some(ref mut player) = gamestate.player {
                                    player.set_state(SpriteState::MovingLeft);
                                }
                            }
                            GameAction::MoveRight => {
                                if let Some(ref mut player) = gamestate.player {
                                    player.set_state(SpriteState::MovingRight);
                                }
                            }
                            GameAction::Shoot => {
                                if let Some(ref mut player) = gamestate.player {
                                    player.action(SpriteAction::Shoot);
                                }
                            }
                            GameAction::StopMoving => {
                                if let Some(ref mut player) = gamestate.player {
                                    player.set_state(SpriteState::Idle);
                                }
                            }
                        }
                    }
                }
            }
            
            let mut player = gamestate.player.clone();
    
            if let Some(ref p) = player {
                p.player_lives_display(ctx, gameboard);
            }
            
            player.as_mut().map(|p| p.react(ctx, gameboard));
            
            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            gamestate.player = player;

            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            let mut enemies = gamestate.enemies.clone();
            
            if enemies.is_empty() {
                let patterns = [
                    EnemyPatterns::Star,
                    EnemyPatterns::Triangle,
                    EnemyPatterns::Circle,
                ];
                let mut rng = thread_rng();
                let pattern = &patterns[rng.gen_range(0..patterns.len())];
                let new_enemies = pattern.get(ctx, gameboard).into_iter()
                    .map(|(s, id)| Enemy::new(ctx, gameboard, s, id)).collect::<Vec<Enemy>>();
                enemies = new_enemies;
            }

            enemies.iter_mut().for_each(|e| e.react(ctx, gameboard));

            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            gamestate.enemies = enemies;

            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            let mut bullets = gamestate.bullets.clone();

            bullets.retain_mut(|b| b.react(ctx, gameboard));

            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            gamestate.bullets = bullets;

            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            let mut explosions = gamestate.explosions.clone();

            explosions.retain_mut(|e| e.react(ctx, gameboard));
            
            let gamestate = ctx.state().get_mut_or_default::<GameState>();
            gamestate.explosions = explosions;

            let (maxw, maxh) = gameboard.0.size(ctx);
            gameboard.2.iter_mut().enumerate().for_each(|(i, s)| {
                if let Some(location) = gameboard.0.0.get_mut(i+1) {
                    let (x, y) = s.position(ctx);
                    location.0 = Offset::Static(x);
                    location.1 = Offset::Static(y);
                }
            });

        } else if let Some(CollisionEvent(a, b)) = event.downcast_ref::<CollisionEvent>() {
            let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
            println!("{:?} collided into {:?}", b, a);
            if a.starts_with("player") && b.starts_with("missile") { // enemy bullet hit player ship
                gamestate.bullets.retain_mut(|bu| bu.id() != *b);
                gameboard.remove_sprite_by_id(b);
                gamestate.player.as_mut().map(|p| p.action(SpriteAction::Hurt));
                
            } else if a.starts_with("enemy") && b.starts_with("bullet") { // player bullet hit enemy ship
                gamestate.bullets.retain_mut(|bu| bu.id() != *b);
                gameboard.remove_sprite_by_id(b);

                let enemy = gameboard.get_sprite_by_id(a).unwrap();
                let pos = enemy.position(ctx);

                let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
                gamestate.enemies.retain_mut(|e| e.id() != *a);
                gameboard.remove_sprite_by_id(a);

                let explosion = Explosion::new(ctx, gameboard, pos.0, pos.1);
                let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
                gamestate.explosions.push(explosion);
            }
        } else if let Some(keyboard_event) = event.downcast_ref::<KeyboardEvent>() {
            // Keep keyboard controls as backup/alternative input
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

#[derive(Debug, Component)]
pub struct Settings(Stack, Page);
impl OnEvent for Settings {}

impl AppPage for Settings {
    fn has_nav(&self) -> bool {false}
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
        match index {
            0 => Ok(Box::new(Galaga::new(ctx))),
            _ => Err(self)
        }
    }
}

impl Settings{
    pub fn new(ctx: &mut Context) -> Self {
        let buttons = vec
        let content = Content::new(ctx, Offset::Start, vec![]);
        let settings = IconButton::navigation(ctx, "settings", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let header = Header::stack(ctx, None, "Galaga", Some(settings));
        Galaga(Column::center(24.0), header, gameboard)
    }
}