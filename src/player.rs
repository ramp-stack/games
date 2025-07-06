use std::any::Any;
use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding, Column, Wrap, Row, ButtonSize, ButtonWidth, ButtonStyle, ButtonState, IconButton, NavigateEvent, DataItem};
use pelican_game_engine::{AspectRatio, Sprite, Gameboard, SpriteState, SpriteAction};

use crate::npcs::Bullet;
use crate::galaga::GameState;

#[derive(Default, Debug, Clone)]
pub struct Player(SpriteState, Vec<SpriteAction>);

impl Player {
    pub fn new(ctx: &mut Context, gameboard: &mut Gameboard) -> Self {
        let player = Sprite::new(ctx, "player", "spaceship", (50.0, 50.0), (Offset::Center, Offset::End));
        gameboard.insert_sprite(ctx, player);
        Player(SpriteState::Idle, Vec::new())
    }

    pub fn react(&mut self, ctx: &mut Context, gameboard: &mut Gameboard) {
        // println!("Player is reacting with state {:?} and actions {:?}", self.0, self.1);
        let player = gameboard.get_sprite_by_id("player").unwrap();
        match self.0 {
            SpriteState::Idle => {},
            SpriteState::MovingLeft => player.adjustments().0 -= 1.0,
            SpriteState::MovingRight => player.adjustments().0 += 1.0,
            _ => {}
        }

        let pos = player.position(ctx).clone();
        let dim = player.dimensions().clone();

        self.1.retain_mut(|a| {
            match a {
                SpriteAction::Hurt => false,
                SpriteAction::Die => false,
                SpriteAction::Shoot => {
                    let bullet = Bullet::new(ctx, gameboard, SpriteState::MovingUp, pos.0 + ((dim.0/2.0) - 7.5), pos.1 - 20.0);
                    let gamestate = ctx.state().get_mut_or_default::<GameState>();
                    gamestate.bullets.push(bullet);
                    false
                },
                _ => true,
            }
        });

    }

    pub fn set_state(&mut self, state: SpriteState) {
        self.0 = state;
    }

    pub fn action(&mut self, action: SpriteAction) {
        self.1.push(action);
    }
}