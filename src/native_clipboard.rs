extern crate clipboard;

use crate::WordleShare;
use bevy::prelude::*;
use clipboard::{ClipboardContext, ClipboardProvider};

pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, paste);
    }
}

fn paste(keyboard_input: Res<Input<KeyCode>>, mut wordle_share: ResMut<WordleShare>) {
    let ctrl_pressed = keyboard_input.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::SuperLeft,
        KeyCode::ControlRight,
        KeyCode::SuperRight,
    ]);

    if ctrl_pressed && keyboard_input.just_pressed(KeyCode::V) {
        let context: Result<ClipboardContext, _> = ClipboardProvider::new();

        if let Ok(mut context) = context {
            if let Ok(contents) = context.get_contents() {
                wordle_share.0 = contents;
            }
        }
    }
}
