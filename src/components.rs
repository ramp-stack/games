// use std::any::Any;
// use std::arch::aarch64::__crc32b;
// use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
// use pelican_ui::drawable::{Align, Drawable, Component};
// use pelican_ui::layout::{Area, SizeRequest, Layout};
// use pelican_ui::{Context, Component};
// use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding, Column, Wrap, Row, ButtonSize, ButtonWidth, ButtonStyle, ButtonState, IconButton, NavigateEvent, DataItem};
// use pelican_game_engine::{AspectRatio, Sprite, Gameboard, SpriteAction};

// pub struct SettingsButton;
// impl SettingsButton {
//     pub fn new (
//         ctx: &mut Context,
//         label: &str,
//         description: &str,
//         buttons: Vec<(&'static str, &str, Box<dyn FnMut(&mut Context)>)>,
//     ) -> Box<dyn Drawable> {
//         let buttons = buttons.into_iter().map(|(i, l, c)| {
//             Button::secondary(ctx, Some(i), l, None, c)
//         }).collect();

//         Box::new(DataItem::new(ctx, None, label, None, Some(description), None, Some(buttons)))
//     }
// }
