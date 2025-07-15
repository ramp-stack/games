use std::any::Any;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::__crc32b;
use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding, Column, Wrap, Row, ButtonSize, ButtonWidth, ButtonStyle, ButtonState, IconButton, NavigateEvent, DataItem};
use pelican_game_engine::{AspectRatio, Sprite, Gameboard, SpriteState, SpriteAction, CollisionEvent};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use crate::game_collection::galaga_game::events::{AdjustPressureEvent, ToggleFliesShoot, ToggleAutoMove, ToggleAutoShoot, ToggleInvincibility};
use crate::game_collection::galaga_game::galaga::{GameState, Galaga};

#[derive(Debug, Component)]
pub struct Settings(Stack, Page, #[skip] Option<Gameboard>);

impl OnEvent for Settings {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(AdjustPressureEvent(p)) = event.downcast_ref::<AdjustPressureEvent>() {
            let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
            if gamestate.peak_min < 1000.0 {
                gamestate.peak_min += *p as f64; 
                println!("peak: {}", gamestate.peak_min);
                *self.1.content().find_at::<DataItem>(0).unwrap().label() = format!("Touchpad Pressure: {:.0}", gamestate.peak_min);
            }
        } else if event.downcast_ref::<ToggleFliesShoot>().is_some() {
            let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
            gamestate.can_shoot = !gamestate.can_shoot;
            let val = if gamestate.can_shoot {"Yes"} else {"No"};
            *self.1.content().find_at::<DataItem>(1).unwrap().label() = format!("Enemy Flies Can Shoot: {}", val);
            let buttons = &mut self.1.content().find_at::<DataItem>(1).unwrap().buttons();
            let label = &mut buttons.as_mut().unwrap()[0].label().as_mut().unwrap().text().spans[0].text;
            *label = if gamestate.can_shoot { "Turn Off".to_string() } else { "Turn On".to_string() };
        } else if event.downcast_ref::<ToggleAutoMove>().is_some() {
            let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
            gamestate.player_auto_move = !gamestate.player_auto_move;
            let val = if gamestate.player_auto_move {"Yes"} else {"No"};
            *self.1.content().find_at::<DataItem>(2).unwrap().label() = format!("Player Auto Moves: {}", val);
            let buttons = &mut self.1.content().find_at::<DataItem>(2).unwrap().buttons();
            let label = &mut buttons.as_mut().unwrap()[0].label().as_mut().unwrap().text().spans[0].text;
            *label = if gamestate.player_auto_move { "Turn Off".to_string() } else { "Turn On".to_string() };
        } else if event.downcast_ref::<ToggleAutoShoot>().is_some() {
            let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
            gamestate.player_auto_shoot = !gamestate.player_auto_shoot;
            let val = if gamestate.player_auto_shoot {"Yes"} else {"No"};
            *self.1.content().find_at::<DataItem>(3).unwrap().label() = format!("Player Auto Shoots: {}", val);
            let buttons = &mut self.1.content().find_at::<DataItem>(3).unwrap().buttons();
            let label = &mut buttons.as_mut().unwrap()[0].label().as_mut().unwrap().text().spans[0].text;
            *label = if gamestate.player_auto_shoot { "Turn Off".to_string() } else { "Turn On".to_string() };
        } else if event.downcast_ref::<ToggleInvincibility>().is_some() {
            let gamestate = &mut ctx.state().get_mut_or_default::<GameState>();
            gamestate.player_invincible = !gamestate.player_invincible;
            let val = if gamestate.player_invincible {"Yes"} else {"No"};
            *self.1.content().find_at::<DataItem>(4).unwrap().label() = format!("Player Is Invincible: {}", val);
            let buttons = &mut self.1.content().find_at::<DataItem>(4).unwrap().buttons();
            let label = &mut buttons.as_mut().unwrap()[0].label().as_mut().unwrap().text().spans[0].text;
            *label = if gamestate.player_invincible { "Turn Off".to_string() } else { "Turn On".to_string() };
        }
        true
    }
}

impl AppPage for Settings {
    fn has_nav(&self) -> bool {false}
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
        match index {
            0 => Ok(Box::new(Galaga::new(ctx, Some(self.2.take().unwrap())))),
            _ => Err(self)
        }
    }
}

impl Settings {
    pub fn new(ctx: &mut Context, gameboard: Gameboard) -> Self {
        let (pressure, can_shoot, auto_move, auto_shoot, invincible, can_shoot_toggle, auto_move_toggle, auto_shoot_toggle, invincible_toggle) = {
            let gamestate = ctx.state().get_mut_or_default::<GameState>();
            let pressure = format!("Touchpad Pressure: {:.0}", gamestate.peak_min);
            let can_shoot = format!("Enemy Flies Can Shoot: {}", if gamestate.can_shoot {"Yes"} else {"No"});
            let auto_move = format!("Player Auto Moves: {}", if gamestate.player_auto_move {"Yes"} else {"No"});
            let auto_shoot = format!("Player Auto Shoots: {}", if gamestate.player_auto_shoot {"Yes"} else {"No"});
            let invincible = format!("Player Is Invincible: {}", if gamestate.player_invincible {"Yes"} else {"No"});
            
            let can_shoot_toggle = if gamestate.can_shoot { "Turn Off" } else { "Turn On" };
            let auto_move_toggle = if gamestate.player_auto_move { "Turn Off" } else { "Turn On" };
            let auto_shoot_toggle = if gamestate.player_auto_shoot { "Turn Off" } else { "Turn On" };
            let invincible_toggle = if gamestate.player_invincible { "Turn Off" } else { "Turn On" };
            
            (pressure, can_shoot, auto_move, auto_shoot, invincible, can_shoot_toggle, auto_move_toggle, auto_shoot_toggle, invincible_toggle)
        };

        let buttons = vec![
            DataItemSettings::new(ctx, &pressure, "Increase or decrease pressure required to perform an action.", vec![
                ("add", "Decrease", Box::new(|ctx: &mut Context| ctx.trigger_event(AdjustPressureEvent(-50.0))) as Box<dyn FnMut(&mut Context)>),
                ("add", "Increase", Box::new(|ctx: &mut Context| ctx.trigger_event(AdjustPressureEvent(50.0))) as Box<dyn FnMut(&mut Context)>),
            ]),
            DataItemSettings::new(ctx, &can_shoot, "Allows enemy flies to shoot back.", vec![
                ("add", can_shoot_toggle, Box::new(|ctx: &mut Context| ctx.trigger_event(ToggleFliesShoot)) as Box<dyn FnMut(&mut Context)>)
            ]),
            DataItemSettings::new(ctx, &auto_move, "Allows player to move back and forth automatically.", vec![
                ("add", auto_move_toggle, Box::new(|ctx: &mut Context| ctx.trigger_event(ToggleAutoMove)) as Box<dyn FnMut(&mut Context)>)
            ]),
            DataItemSettings::new(ctx, &auto_shoot, "Allows player to automatically shoot every 200 millis.", vec![
                ("add", auto_shoot_toggle, Box::new(|ctx: &mut Context| ctx.trigger_event(ToggleAutoShoot)) as Box<dyn FnMut(&mut Context)>)
            ]),
            DataItemSettings::new(ctx, &invincible, "Allows player to be invincible to enemy fire.", vec![
                ("add", invincible_toggle, Box::new(|ctx: &mut Context| ctx.trigger_event(ToggleInvincibility)) as Box<dyn FnMut(&mut Context)>)
            ]),
        ];

        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));

        let header = Header::stack(ctx, Some(back), "Settings", None);
        // Convert Vec<DataItem> to Vec<Box<dyn Drawable>>
        let drawable_buttons: Vec<Box<dyn Drawable>> = buttons.into_iter().map(|item| Box::new(item) as Box<dyn Drawable>).collect();
        let content = Content::new(Offset::Start, drawable_buttons);

        Settings(Stack::default(), Page::new(Some(header), content, None), Some(gameboard))
    }
}

pub struct DataItemSettings;

impl DataItemSettings {
    pub fn new(ctx: &mut Context, title: &str, sub: &str, buttons: Vec<(&'static str, &str, Box<dyn FnMut(&mut Context)>)>) -> DataItem {
        let buttons = buttons.into_iter().map(|(i, n, c)| Button::secondary(ctx, Some(i), n, None, c, None)).collect::<Vec<_>>();
        DataItem::new(ctx, None, title, Some(sub), None, None, Some(buttons))
    }
}