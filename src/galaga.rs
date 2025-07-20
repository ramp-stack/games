use std::any::Any;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::__crc32b;
use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, ExpandableText, TextStyle, Text, AppPage, Size, Padding, Column, Wrap, Row, ButtonSize, ButtonWidth, ButtonStyle, ButtonState, IconButton, NavigateEvent, DataItem};
use pelican_game_engine::{AspectRatio, Sprite, Gameboard, SpriteState, SpriteAction, CollisionEvent};

use crate::ArduinoServer;
use crate::player::Player;
use crate::npcs::{Enemy, EnemyPatterns, Bullet, Explosion};
use crate::server::{GameAction, ActionEvent, SimultaneousActions}; 
use crate::settings::Settings;

use std::time::Instant;
use tokio::time::Instant as TokioInstant;
use rand::thread_rng;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

#[derive(Default, Clone)]
pub struct GameState {
    pub player: Option<Player>,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub explosions: Vec<Explosion>,
    pub interval: Option<Instant>,
    pub arduino_server: Option<ArduinoServer>,
    pub action_queue: Option<Arc<Mutex<VecDeque<ActionEvent>>>>,
    pub peak_min: f64,
    pub can_shoot: bool,
    pub player_auto_move: bool,
    pub player_auto_shoot: bool,
    pub player_invincible: bool,
    pub score: u32,
    pub current_movement_state: Option<GameAction>,
    pub last_movement_update: Option<Instant>,
}

impl std::fmt::Debug for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameState")
            .finish()
    }
}

impl GameState {
    pub fn new() -> Self {
        let arduino_server = ArduinoServer::new();
        let action_queue = arduino_server.get_action_queue();
        
        let initial_threshold = 200.0;
        arduino_server.set_pressure_threshold(initial_threshold);
        
        let server_clone = arduino_server.clone();
        tokio::spawn(async move {
            server_clone.start().await;
        });
   
        GameState {
            player: None,
            enemies: Vec::new(),
            bullets: Vec::new(),
            explosions: Vec::new(),
            interval: Some(Instant::now()),
            arduino_server: Some(arduino_server),
            action_queue: Some(action_queue),
            peak_min: initial_threshold, 
            can_shoot: true,
            player_auto_move: false,
            player_auto_shoot: false,
            player_invincible: false,
            score: 0,
            current_movement_state: None,
            last_movement_update: None,
        }
    }

    pub fn set_action_queue(&mut self, queue: Arc<Mutex<VecDeque<ActionEvent>>>) {
        self.action_queue = Some(queue);
    }
    
    pub fn sync_pressure_threshold(&mut self) {
        if let Some(ref arduino_server) = self.arduino_server {
            arduino_server.set_pressure_threshold(self.peak_min);
        }
    }
}
#[derive(Debug, Component)]
pub struct Galaga(Column, Header, ExpandableText, Option<Gameboard>);

impl OnEvent for Galaga {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            let score = format!("SCORE: {}", gamestate.score);
            self.2.text().spans[0].text = score;
        }
        true
    }
}

impl AppPage for Galaga {
    fn has_nav(&self) -> bool {false}
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
        match index {
            0 => Ok(Box::new(Settings::new(ctx, self.3.take().unwrap()))),
            _ => Err(self)
        }
    }
}

impl Galaga {
    pub fn new(ctx: &mut Context, gameboard: Option<Gameboard>) -> Self {
        let mut gameboard = gameboard.unwrap_or(Gameboard::new(ctx, AspectRatio::OneOne, Box::new(Self::on_event)));

        let mut gamestate = match ctx.state().get::<GameState>() {
            Some(state) => state.clone(),
            None => {
                let mut state = GameState::new();
                let mut player = Player::new(ctx, &mut gameboard);
        
                player.set_auto_shoot(false);
                player.set_auto_move(false);
                
                player.player_lives_display(ctx, &mut gameboard);
                
                state.player = Some(player);
                state
            }
        };
        let score = gamestate.score.to_string();
        ctx.state().set(gamestate);
        let settings = IconButton::navigation(ctx, "settings", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let header = Header::stack(ctx, None, "Galaga", Some(settings));
        let text_size = ctx.theme.fonts.size.h3;
        let score = format!("SCORE: {}", score);
        let text = ExpandableText::new(ctx, &score, TextStyle::Heading, text_size, Align::Center, None);
        Galaga(Column::center(24.0), header, text, Some(gameboard))
    }

    fn on_event(gameboard: &mut Gameboard, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let mut gamestate = ctx.state().get_mut_or_default::<GameState>();
            let current_time = Instant::now();
            
            if let Some(ref arduino_server) = gamestate.arduino_server.clone() {
                let simultaneous_actions = arduino_server.get_simultaneous_actions();
                
                if !simultaneous_actions.is_empty() {
                    let mut latest_actions = std::collections::HashMap::new();
                    for action in simultaneous_actions {
                        latest_actions.insert(action.connection_id.clone(), action);
                    }
                    
                    for (connection_id, action) in latest_actions {
                        match action.movement {
                            Some(GameAction::MoveLeft) => {
                                gamestate.current_movement_state = Some(GameAction::MoveLeft);
                                gamestate.last_movement_update = Some(current_time);
                            }
                            Some(GameAction::MoveRight) => {
                                gamestate.current_movement_state = Some(GameAction::MoveRight);
                                gamestate.last_movement_update = Some(current_time);
                            }
                            None => {
                            }
                            _ => {}
                        }
                        
                        if let Some(ref mut player) = gamestate.player {
                            if action.shooting {
                                player.action(SpriteAction::Shoot);
                            }
                        }
                    }
                }
            }
            
            if let Some(last_update) = gamestate.last_movement_update {
                if current_time.duration_since(last_update).as_millis() > 200 {
                    gamestate.current_movement_state = None;
                }
            }
            
            if let Some(ref mut player) = gamestate.player {
                match gamestate.current_movement_state {
                    Some(GameAction::MoveLeft) => {
                        player.set_state(SpriteState::MovingLeft);
                    }
                    Some(GameAction::MoveRight) => {
                        player.set_state(SpriteState::MovingRight);
                    }
                    None => {
                        player.set_state(SpriteState::Idle);
                    }
                    _ => {}
                }
            }
            
            if let Some(ref action_queue) = gamestate.action_queue.clone() {
                if let Ok(mut queue) = action_queue.lock() {
                    while let Some(action_event) = queue.pop_front() {
                        match action_event.action {
                            GameAction::MoveLeft => {
                                gamestate.current_movement_state = Some(GameAction::MoveLeft);
                                gamestate.last_movement_update = Some(current_time);
                            }
                            GameAction::MoveRight => {
                                gamestate.current_movement_state = Some(GameAction::MoveRight);
                                gamestate.last_movement_update = Some(current_time);
                            }
                            GameAction::Shoot => {
                                if let Some(ref mut player) = gamestate.player {
                                    player.action(SpriteAction::Shoot);
                                }
                            }
                            GameAction::Idle => {
                                gamestate.current_movement_state = None;
                            }
                        }
                    }
                }
            }
            
            let mut player = gamestate.player.clone();
    
            player.as_mut().map(|p| p.player_lives_display(ctx, gameboard));
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
            if a.starts_with("player") && b.starts_with("missile") && !gamestate.player_invincible {
                gamestate.bullets.retain_mut(|bu| bu.id() != *b);
                gameboard.remove_sprite_by_id(b);
                gamestate.player.as_mut().map(|p| p.action(SpriteAction::Hurt));
            } else if a.starts_with("missile") && b.starts_with("player") && !gamestate.player_invincible {
                gamestate.bullets.retain_mut(|bu| bu.id() != *a);
                gameboard.remove_sprite_by_id(a);
                gamestate.player.as_mut().map(|p| p.action(SpriteAction::Hurt));
            } else if a.starts_with("enemy") && b.starts_with("bullet") {
                gamestate.score += 250;
                gamestate.bullets.retain_mut(|bu| bu.id() != *b);
                gameboard.remove_sprite_by_id(b);

                if let Some(enemy) = gameboard.get_sprite_by_id(a) {
                    let pos = enemy.position(ctx);
                    let dim = enemy.dimensions().clone();

                    let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
                    gamestate.enemies.retain_mut(|e| e.id() != *a);
                    gameboard.remove_sprite_by_id(a);

                    let explosion = Explosion::new(ctx, gameboard, pos, dim);
                    let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
                    gamestate.explosions.push(explosion);
                }
            } else if a.starts_with("bullet") && b.starts_with("enemy") {
                gamestate.score += 250;
                gamestate.bullets.retain_mut(|bu| bu.id() != *a);
                gameboard.remove_sprite_by_id(a);

                if let Some(enemy) = gameboard.get_sprite_by_id(b) {
                    let pos = enemy.position(ctx);
                    let dim = enemy.dimensions().clone();

                    let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
                    gamestate.enemies.retain_mut(|e| e.id() != *b);
                    gameboard.remove_sprite_by_id(b);

                    let explosion = Explosion::new(ctx, gameboard, pos, dim);
                    let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
                    gamestate.explosions.push(explosion);
                }
            }else if a.starts_with("bullet") && b.starts_with("missile") || b.starts_with("bullet") && a.starts_with("missile"){
                if let Some(bullet) = gameboard.get_sprite_by_id(b) {
                    let pos = bullet.position(ctx).clone();
                    let dim = bullet.dimensions().clone();
                    let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();

                    gamestate.bullets.retain_mut(|bu| bu.id() != *a);
                    gameboard.remove_sprite_by_id(a);
                    gamestate.bullets.retain_mut(|bu| bu.id() != *b);
                    gameboard.remove_sprite_by_id(b);

                    let explosion = Explosion::new(ctx, gameboard, pos, dim);
                    let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
                    gamestate.explosions.push(explosion);
                }
            }
        } else if let Some(keyboard_event) = event.downcast_ref::<KeyboardEvent>() {
            let gamestate = ctx.state().get_mut_or_default::<GameState>();
            let current_time = Instant::now();
            
            match keyboard_event {
                KeyboardEvent { state: KeyboardState::Pressed, key: Key::Named(NamedKey::ArrowLeft) } => {
                    gamestate.current_movement_state = Some(GameAction::MoveLeft);
                    gamestate.last_movement_update = Some(current_time);
                }
                KeyboardEvent { state: KeyboardState::Released, key: Key::Named(NamedKey::ArrowLeft) } => {
                    if gamestate.current_movement_state == Some(GameAction::MoveLeft) {
                        gamestate.current_movement_state = None;
                    }
                }
                KeyboardEvent { state: KeyboardState::Pressed, key: Key::Named(NamedKey::ArrowRight) } => {
                    gamestate.current_movement_state = Some(GameAction::MoveRight);
                    gamestate.last_movement_update = Some(current_time);
                }
                KeyboardEvent { state: KeyboardState::Released, key: Key::Named(NamedKey::ArrowRight) } => {
                    if gamestate.current_movement_state == Some(GameAction::MoveRight) {
                        gamestate.current_movement_state = None;
                    }
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