use std::any::Any;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::__crc32b;
use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding, Column, Wrap, Row, ButtonSize, ButtonWidth, ButtonStyle, ButtonState, IconButton, NavigateEvent, DataItem};
use pelican_game_engine::{AspectRatio, Sprite, Gameboard, SpriteState, SpriteAction, CollisionEvent};

use crate::events::{AdjustPressureEvent, ToggleFliesShoot, ToggleAutoMove, ToggleAutoShoot, ToggleInvincibility};

use crate::galaga::GameState;

#[derive(Debug, Component)]
pub struct Settings(Stack, Page);
impl OnEvent for Settings {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(AdjustPressureEvent(p)) = event.downcast_ref::<AdjustPressureEvent>() {
            let mut peak = &mut ctx.state().get_mut_or_default::<GameState>().peak_min;
            if  *peak < 1000.0 {
                ctx.state().get_mut_or_default::<GameState>().peak_min += p;

                println!("peak: {}",  ctx.state().get_mut_or_default::<GameState>().peak_min);

                *self.1.content().find_at::<DataItem>(0).unwrap().label() = format!("Touchpad Pressure: {:.0}", ctx.state().get_mut_or_default::<GameState>().unwrap().peak_min);
            }
        } else if event.downcast_ref::<ToggleFliesShoot>().is_some() {
            let can_shoot = !ctx.state().get_mut_or_default::<GameState>().can_shoot;
            ctx.state().get_mut_or_default::<GameState>().can_shoot = can_shoot;
            let val = if can_shoot {"Yes"} else {"No"};
            *self.1.content().find_at::<DataItem>(1).unwrap().label() = format!("Enemy Flies Can Shoot: {}", val);
            let buttons = &mut self.1.content().find_at::<DataItem>(1).unwrap().buttons();
            let label = &mut buttons.as_mut().unwrap()[0].label().as_mut().unwrap().text().spans[0].text;
            *label = if can_shoot { "Turn Off".to_string() } else { "Turn On".to_string() };
        } else if event.downcast_ref::<ToggleAutoMove>().is_some() {
            let player_auto_move = !ctx.state().get_mut_or_default::<GameState>().player_auto_move;
            ctx.state().get_mut_or_default::<GameState>().player_auto_move = player_auto_move;
            let val = if player_auto_move {"Yes"} else {"No"};
            *self.1.content().find_at::<DataItem>(2).unwrap().label() = format!("Player Auto Moves: {}", val);
            let buttons = &mut self.1.content().find_at::<DataItem>(2).unwrap().buttons();
            let label = &mut buttons.as_mut().unwrap()[0].label().as_mut().unwrap().text().spans[0].text;
            *label = if player_auto_move { "Turn Off".to_string() } else { "Turn On".to_string() };
        } else if event.downcast_ref::<ToggleAutoShoot>().is_some() {
            let player_auto_shoot = !ctx.state().get_mut_or_default::<GameState>().player_auto_shoot;
            ctx.state().get_mut_or_default::<GameState>().player_auto_shoot = player_auto_shoot;
            let val = if player_auto_shoot {"Yes"} else {"No"};
            *self.1.content().find_at::<DataItem>(3).unwrap().label() = format!("Player Auto Shoots: {}", val);
            let buttons = &mut self.1.content().find_at::<DataItem>(3).unwrap().buttons();
            let label = &mut buttons.as_mut().unwrap()[0].label().as_mut().unwrap().text().spans[0].text;
            *label = if player_auto_shoot { "Turn Off".to_string() } else { "Turn On".to_string() };
        } else if event.downcast_ref::<ToggleInvincibility>().is_some() {
            let player_invincible = !ctx.state().get_mut_or_default::<GameState>().player_invincible;
            ctx.state().get_mut_or_default::<GameState>().player_invincible = player_invincible;
            let val = if player_invincible {"Yes"} else {"No"};
            *self.1.content().find_at::<DataItem>(4).unwrap().label() = format!("Player Auto Shoots: {}", val);
            let buttons = &mut self.1.content().find_at::<DataItem>(4).unwrap().buttons();
            let label = &mut buttons.as_mut().unwrap()[0].label().as_mut().unwrap().text().spans[0].text;
            *label = if player_invincible { "Turn Off".to_string() } else { "Turn On".to_string() };
        }
        true
    }
}

impl AppPage for Settings {
    fn has_nav(&self) -> bool {false}
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
        match index {
            0 => Ok(Box::new(Galaga::new(ctx))),
            _ => Err(self)
        }
    }
}


impl Settings {
    pub fn new(ctx: &mut Context) -> Self {
        let pressure = format!("Touchpad Pressure: {:.0}", ctx.state().get_mut_or_default::<GameState>().unwrap().peak_min);
        let can_shoot = format!("Enemy Flies Can Shoot: {}", if ctx.state().get_mut_or_default::<GameState>().unwrap().can_shoot {"Yes"} else {"No"});
        let auto_move = format!("Player Auto Moves: {}", if ctx.state().get_mut_or_default::<GameState>().unwrap().player_auto_move {"Yes"} else {"No"});
        let auto_shoot = format!("Player Auto Shoots: {}", if ctx.state().get_mut_or_default::<GameState>().unwrap().player_auto_shoot {"Yes"} else {"No"});
        let invincible = format!("Player Is Invincible: {}", if ctx.state().get_mut_or_default::<GameState>().unwrap().player_invincible {"Yes"} else {"No"});

        let buttons = vec![
            DataItemSettings::new(ctx, &pressure, "Increase or decrease pressure required to perform an action.", vec![
                ("add", "Decrease", Box::new(|ctx: &mut Context| ctx.trigger_event(AdjustPressureEvent(-50.0))) as Box<dyn FnMut(&mut Context)>),
                ("add", "Increase", Box::new(|ctx: &mut Context| ctx.trigger_event(AdjustPressureEvent(50.0))) as Box<dyn FnMut(&mut Context)>),
            ]),
            DataItemSettings::new(ctx, &can_shoot, "Allows enemy flies to shoot back.", vec![
                ("add", "Turn Off", Box::new(|ctx: &mut Context| ctx.trigger_event(ToggleFliesShoot)) as Box<dyn FnMut(&mut Context)>)
            ]),
            DataItemSettings::new(ctx, &auto_move, "Allows player to move back and forth automatically.", vec![
                ("add", "Turn On", Box::new(|ctx: &mut Context| ctx.trigger_event(ToggleAutoMove)) as Box<dyn FnMut(&mut Context)>)
            ]),
            DataItemSettings::new(ctx, &auto_shoot, "Allows player to automatically shoot every 200 millis.", vec![
                ("add", "Turn On", Box::new(|ctx: &mut Context| ctx.trigger_event(ToggleAutoShoot)) as Box<dyn FnMut(&mut Context)>)
            ]),
            DataItemSettings::new(ctx, &invincible, "Allows player to be invincible to enemy fire.", vec![
                ("add", "Turn On", Box::new(|ctx: &mut Context| ctx.trigger_event(ToggleInvincibility)) as Box<dyn FnMut(&mut Context)>)
            ]),
        ];

        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));

        let header = Header::stack(ctx, Some(back), "Settings", None);
        let content = Content::new(Offset::Start, buttons as Vec<Box<dyn Drawable>);

        Settings(Stack::default(), Page::new(Some(header), content, None))
    }
}

pub struct DataItemSettings;

impl DataItemSettings {
    pub fn new(ctx: &mut Context, title: &str, sub: &str, buttons: Vec<(&'static str, &str, Box<dyn FnMut(&mut Context)>)>) -> DataItem {
        let buttons = buttons.into_iter().map(|(i, n, c)| Button::secondary(ctx, Some(i), n, None, c, None)).collect::<Vec<_>>();
        DataItem::new(ctx, None, "Bitcoin address", Some(address), None, None, Some(buttons))
    }
}