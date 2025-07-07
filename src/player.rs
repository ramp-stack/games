use std::any::Any;
use std::time::Instant;
use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding, Column, Wrap, Row, ButtonSize, ButtonWidth, ButtonStyle, ButtonState, IconButton, NavigateEvent, DataItem};
use pelican_game_engine::{AspectRatio, Sprite, Gameboard, SpriteState, SpriteAction};
use crate::npcs::Explosion;

use crate::npcs::Bullet;
use crate::galaga::GameState;

#[derive(Default, Debug, Clone)]
// Fields: SpriteState, Vec<SpriteAction>, lives (u32), auto_shoot_timer (Option<Instant>), auto_move_enabled (bool), auto_move_direction (bool - true=right, false=left)
pub struct Player(SpriteState, Vec<SpriteAction>, u32, Option<Instant>, bool, bool, Option<Instant>);

impl Player {
    //Sprite::new requires ctx, an id for the image, an id for the png file, the offset numbers, and the offset positions.
    pub fn new(ctx: &mut Context, gameboard: &mut Gameboard) -> Self {
        let player = Sprite::new(ctx, "player", "spaceship", (50.0, 50.0), (Offset::Center, Offset::End));
        //gameboard is the container of everything- it has a width and height essentially
        gameboard.insert_sprite(ctx, player);

        //give the state of the sprite, we construct Vec, and we give the lives.  
        Player(SpriteState::Idle, Vec::new(), 3, None, false, false, None)
    }

    pub fn player_lives_display(&self, ctx: &mut Context, gameboard: &mut Gameboard) {
        for i in 0..5 {
            gameboard.remove_sprite_by_id(&format!("player_life_{}", i));
        }
        
        for i in 0..self.2 {
            let life_sprite = Sprite::new(
                ctx,
                &format!("player_life_{}", i),
                "spaceship",
                (40.0, 40.0), 
                (Offset::Static(20.0 + i as f32 * 50.0), Offset::Static(10.0)), 
            );
            gameboard.insert_sprite(ctx, life_sprite);
        }
    }
    
    pub fn set_auto_shoot(&mut self, enable: bool) {
        if enable {
            self.3 = Some(Instant::now());
        } else {
            self.3 = None;
        }
    }

    pub fn set_auto_move(&mut self, enable: bool) {
        self.4 = enable;
        if enable {
            self.5 = false; // Start moving left
        }
    }

    
    pub fn react(&mut self, ctx: &mut Context, gameboard: &mut Gameboard) {
        // println!("Player is reacting with state {:?} and actions {:?}", self.0, self.1);
        
        // Check if we need to respawn the player after delay
        if let Some(respawn_time) = self.6 {
            if respawn_time.elapsed().as_secs() >= 2 {
                // Respawn the player
                let new_player = Sprite::new(ctx, "player", "spaceship", (50.0, 50.0), (Offset::Center, Offset::End));
                gameboard.insert_sprite(ctx, new_player);
                self.0 = SpriteState::Idle;
                self.6 = None;
                println!("Player respawned after delay!");
            } else {
                // Player is still waiting to respawn, don't process other actions
                return;
            }
        }
        
        let board_width = gameboard.0.size(ctx).0;
        let player_opt = gameboard.get_sprite_by_id("player");
        
        // If player doesn't exist and we're not waiting to respawn, something went wrong
        if player_opt.is_none() && self.6.is_none() {
            return;
        }
        
        let player = player_opt.unwrap();

        if self.4 { 
            let player_pos = player.position(ctx).0;
            let player_width = player.dimensions().0;
            
            if self.5 { 
                if player_pos < board_width - player_width {
                    player.adjustments().0 += 2.0;
                } else {
                    self.5 = false;
                }
            } else { 
                if player_pos > 0.0 {
                    player.adjustments().0 -= 2.0;
                } else {
                    self.5 = true;
                }
            }
        }

        // Handle manual movement only if auto-movement is not active
        if !self.4 {
            match self.0 {
                SpriteState::Idle => {},
                SpriteState::MovingLeft => if player.position(ctx).0 > 0.0 {
                    player.adjustments().0 -= 2.0;
                },
                SpriteState::MovingRight => if player.position(ctx).0 < board_width - player.dimensions().0 {
                    player.adjustments().0 += 2.0;
                },
                _ => {}
            }
        }

        if let Some(last_shot_time) = self.3 {
            if last_shot_time.elapsed().as_millis() > 500 { 
                self.1.push(SpriteAction::Shoot);
                self.3 = Some(Instant::now()); 
            }
        }

        let pos = player.position(ctx).clone();
        let dim = player.dimensions().clone();
        //we create a vec of actions to add 
        let mut actions_to_add = Vec::new();
        //index on spriteactions  
        self.1.retain_mut(|a| {
            match a {
                SpriteAction::Hurt => {
                    let player = gameboard.get_sprite_by_id("player").unwrap();
                    let pos = player.position(ctx);

                    gameboard.remove_sprite_by_id("player");

                    let explosion = Explosion::new(ctx, gameboard, pos.0, pos.1);
                    let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
                    gamestate.explosions.push(explosion);

                    // Set respawn timer instead of immediately respawning
                    self.6 = Some(Instant::now());
                    self.0 = SpriteState::Idle;

                    if self.2 > 0 {
                        self.2 -= 1;
                        println!("Player hurt! Remaining lives: {} (respawning in 2 seconds)", self.2);
                        if self.2 == 0 {
                            println!("Player has died! (respawning in 2 seconds)");
                            actions_to_add.push(SpriteAction::Die);
                        }
                    }
                    false 
                },
                SpriteAction::Die => {
                    gameboard.remove_sprite_by_id("player");
                    
                    self.6 = Some(Instant::now());
    
                    false
                },
                SpriteAction::Shoot => {
                    let bullet = Bullet::new(ctx, gameboard, SpriteState::MovingUp, pos.0 + ((dim.0/2.0) - 7.5), pos.1 - 20.0);
                    let gamestate = ctx.state().get_mut_or_default::<GameState>();
                    gamestate.bullets.push(bullet);
                    false
                },
                _ => true,
            }
        });

        for action in actions_to_add {
            self.1.push(action);
        }
    }

    pub fn set_state(&mut self, state: SpriteState) {
        self.0 = state;
    }

    pub fn action(&mut self, action: SpriteAction) {
        self.1.push(action);
    }
    
    pub fn is_respawning(&self) -> bool {
        self.6.is_some()
    }
}