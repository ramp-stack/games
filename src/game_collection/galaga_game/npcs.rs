use std::any::Any;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::__crc32b;
use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding, Column, Wrap, Row, ButtonSize, ButtonWidth, ButtonStyle, ButtonState, IconButton, NavigateEvent, DataItem};
use pelican_game_engine::{AspectRatio, Sprite, Gameboard, SpriteState, SpriteAction};
use std::time::{Duration, Instant};
use rand::Rng;

use crate::game_collection::galaga_game::galaga::GameState;

#[derive(Default, Debug, Clone)]
pub struct Bullet(SpriteState, String);

impl Bullet {
    pub fn new(ctx: &mut Context, gameboard: &mut Gameboard, state: SpriteState, x: f32, y: f32) -> Self {
        let prefix = if state == SpriteState::MovingUp {"bullet_"} else {"missile_"};
        let c = gameboard.2.iter().filter(|s| s.id().starts_with(prefix)).last().map(|s| s.id().strip_prefix(prefix).unwrap()).unwrap_or("0");
        let id = format!("{}{}", prefix, c.parse::<usize>().unwrap()+1);
        println!("CREATED BULLET {:?}", id);
        let image = if state == SpriteState::MovingUp {"bullet_blue"} else {"bullet_downward"};
        let bullet = Sprite::new(ctx, &id, image, (15.0, 15.0), (Offset::Static(x), Offset::Static(y)));
        gameboard.insert_sprite(ctx, bullet);
        Bullet(state, id)
    }

    pub fn react(&mut self, ctx: &mut Context, gameboard: &mut Gameboard) -> bool {
        let max = gameboard.0.size(ctx).0;
        // println!("Bullet is reacting with state {:?} and id {:?}", self.0, self.1);
        let bullet = gameboard.get_sprite_by_id(&self.1).unwrap();
        match self.0 {
            SpriteState::MovingUp => bullet.adjustments().1 -= 3.0,
            SpriteState::MovingDown => bullet.adjustments().1 += 3.0,
            _ => {}
        }

        let pos = bullet.position(ctx).1;
        if pos > max || pos < 0.0 {
            println!("OUT OF BOUNDS");
            gameboard.remove_sprite_by_id(&self.1);
            return false;
        }
        true
    }

    pub fn id(&self) -> String {self.1.clone()}
}

#[derive(Debug, Clone)]
pub struct Explosion(String, Instant);

impl Default for Explosion {
    fn default() -> Self {
        Explosion(String::new(), Instant::now())
    }
}

impl Explosion {
    pub fn new(ctx: &mut Context, gameboard: &mut Gameboard, pos: (f32, f32), dim: (f32, f32)) -> Self {
        let dim = (dim.0 + 10.0, dim.1 + 10.0);
        let pos = (pos.0 - 5.0, pos.1 - 5.0);
        let c = gameboard.2.iter().filter(|s| s.id().starts_with("explosion")).last().map(|s| s.id().strip_prefix("explosion_").unwrap()).unwrap_or("0");
        let id = format!("explosion_{}", c.parse::<usize>().unwrap()+1);
        println!("CREATED EXPLOSION {:?}", id);
        let explosion = Sprite::new(ctx, &id, "explosion", dim, (Offset::Static(pos.0), Offset::Static(pos.1)));
        gameboard.insert_sprite(ctx, explosion);
        Explosion(id, Instant::now())
    }

    pub fn react(&mut self, ctx: &mut Context, gameboard: &mut Gameboard) -> bool {
        let elapsed = self.1.elapsed();
        if elapsed.as_millis() > 200 {
            gameboard.remove_sprite_by_id(&self.0);
            return false
        }
        true
    }

    pub fn id(&self) -> String {self.0.clone()}
}

#[derive(Default, Debug, Clone)]
pub struct Enemy(SpriteState, Vec<SpriteAction>, String, Duration);

impl Enemy {
    pub fn new(ctx: &mut Context, gameboard: &mut Gameboard, sprite: Sprite, id: String) -> Self {
        gameboard.insert_sprite(ctx, sprite);
        let mut rng = rand::thread_rng();
        let millis = rng.gen_range(500..=1000);
        println!("ENEMY NEW WITH MILIS {:?}", millis);
        Enemy(SpriteState::Idle, Vec::new(), id, Duration::from_millis(millis))
    }

    pub fn react(&mut self, ctx: &mut Context, gameboard: &mut Gameboard) {
        let elapsed = &mut ctx.state().get_mut_or_default::<GameState>().interval.unwrap().elapsed();
        if elapsed.as_millis() % self.3.as_millis() == 0 {
            self.1.push(SpriteAction::Shoot);
        }
        // println!("Enemy is reacting with state {:?} and actions {:?}", self.0, self.1);
        let enemy = gameboard.get_sprite_by_id(&self.2).unwrap();
        match self.0 {
            SpriteState::Idle => {},
            SpriteState::MovingLeft => enemy.adjustments().0 -= 1.0,
            SpriteState::MovingRight => enemy.adjustments().0 += 1.0,
            _ => {}
        }

        let pos = enemy.position(ctx).clone();
        let dim = enemy.dimensions().clone();

        self.1.retain_mut(|a| {
            match a {
                SpriteAction::Hurt => false,
                SpriteAction::Die => false,
                SpriteAction::Shoot => {
                    let gamestate = ctx.state().get_mut_or_default::<GameState>();
                    if gamestate.can_shoot {
                        println!("ENEMY IS SHOOTING");
                        let bullet = Bullet::new(ctx, gameboard, SpriteState::MovingDown, pos.0 + ((dim.0/2.0) - 7.5), pos.1 + 20.0);
                        let gamestate = ctx.state().get_mut_or_default::<GameState>();
                        gamestate.bullets.push(bullet);
                    }
                    false
                },
                _ => true,
            }
        });
    }

    pub fn id(&self) -> String {self.2.clone()}

    pub fn set_state(&mut self, state: SpriteState) {
        self.0 = state;
    }

    pub fn action(&mut self, action: SpriteAction) {
        self.1.push(action);
    }
}

pub enum EnemyType {
    B2,
    TikiFly,
    Northrop,
}

impl EnemyType {
    pub fn get(&self, ctx: &mut Context, c: usize, x: f32, y: f32) -> (Sprite, String) {
        let id = format!("enemy_{}", c);
        let path = match self {
            EnemyType::B2 => "b2",
            EnemyType::TikiFly => "tiki_fly",
            EnemyType::Northrop => "northrop",
        };
        (Sprite::new(ctx, &id, path, (40.0, 40.0), (Offset::Static(x), Offset::Static(y))), id)
    }
}
#[derive(Copy, Clone)]
pub enum EnemyPatterns {
    Star,
    Triangle,
    Circle,
}

impl EnemyPatterns {
    pub fn get(self, ctx: &mut Context, board: &mut Gameboard) -> Vec<(Sprite, String)> {
        let (board_width, board_height) = board.0.size(ctx);
        match self {
            EnemyPatterns::Star => vec![
                EnemyType::B2.get(ctx, 0, board_width * 0.2, board_height * 0.1),
                EnemyType::B2.get(ctx, 1, board_width * 0.4, board_height * 0.1),
                EnemyType::B2.get(ctx, 2, board_width * 0.6, board_height * 0.1),
                EnemyType::B2.get(ctx, 3, board_width * 0.8, board_height * 0.1),
                EnemyType::TikiFly.get(ctx, 4, board_width * 0.15, board_height * 0.2),
                EnemyType::TikiFly.get(ctx, 5, board_width * 0.3, board_height * 0.2),
                EnemyType::TikiFly.get(ctx, 6, board_width * 0.5, board_height * 0.2),
                EnemyType::TikiFly.get(ctx, 7, board_width * 0.7, board_height * 0.2),
                EnemyType::TikiFly.get(ctx, 8, board_width * 0.85, board_height * 0.2),
                EnemyType::Northrop.get(ctx, 9, board_width * 0.25, board_height * 0.3),
                EnemyType::Northrop.get(ctx, 10, board_width * 0.4, board_height * 0.3),
                EnemyType::Northrop.get(ctx, 11, board_width * 0.6, board_height * 0.3),
                EnemyType::Northrop.get(ctx, 12, board_width * 0.75, board_height * 0.3),
            ],
            EnemyPatterns::Triangle => vec![
                EnemyType::B2.get(ctx, 0, board_width * 0.5, board_height * 0.05),
                EnemyType::B2.get(ctx, 1, board_width * 0.3, board_height * 0.15),
                EnemyType::B2.get(ctx, 2, board_width * 0.7, board_height * 0.15),
                EnemyType::TikiFly.get(ctx, 3, board_width * 0.1, board_height * 0.25),
                EnemyType::TikiFly.get(ctx, 4, board_width * 0.5, board_height * 0.25),
                EnemyType::TikiFly.get(ctx, 5, board_width * 0.9, board_height * 0.25),
                EnemyType::Northrop.get(ctx, 6, board_width * 0.2, board_height * 0.35),
                EnemyType::Northrop.get(ctx, 7, board_width * 0.4, board_height * 0.35),
                EnemyType::Northrop.get(ctx, 8, board_width * 0.6, board_height * 0.35),
                EnemyType::Northrop.get(ctx, 9, board_width * 0.8, board_height * 0.35),
            ],
            EnemyPatterns::Circle => vec![
                EnemyType::B2.get(ctx, 0, board_width * 0.1, board_height * 0.1),
                EnemyType::B2.get(ctx, 1, board_width * 0.3, board_height * 0.15),
                EnemyType::B2.get(ctx, 2, board_width * 0.5, board_height * 0.2),
                EnemyType::B2.get(ctx, 3, board_width * 0.7, board_height * 0.15),
                EnemyType::B2.get(ctx, 4, board_width * 0.9, board_height * 0.1),
                EnemyType::TikiFly.get(ctx, 5, board_width * 0.2, board_height * 0.3),
                EnemyType::TikiFly.get(ctx, 6, board_width * 0.4, board_height * 0.25),
                EnemyType::TikiFly.get(ctx, 7, board_width * 0.6, board_height * 0.25),
                EnemyType::TikiFly.get(ctx, 8, board_width * 0.8, board_height * 0.3),
                EnemyType::Northrop.get(ctx, 9, board_width * 0.35, board_height * 0.4),
                EnemyType::Northrop.get(ctx, 10, board_width * 0.65, board_height * 0.4),
            ],
        }
    }
}