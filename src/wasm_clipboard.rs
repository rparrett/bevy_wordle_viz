use crate::WordleShare;
use bevy::prelude::*;
use std::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};

type OnPasteSender = Sender<String>;
type OnPasteReceiver = Receiver<String>;

pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        let channel = std::sync::mpsc::channel();
        let paste_sender: OnPasteSender = channel.0;
        let paste_receiver: OnPasteReceiver = channel.1;

        app.insert_resource(Mutex::new(paste_sender))
            .insert_resource(Mutex::new(paste_receiver))
            .add_startup_system(setup_clipboard_system)
            .add_system(clipboard);
    }
}

fn setup_clipboard_system(paste_sender: Res<Mutex<OnPasteSender>>) {
    let web_window = web_sys::window().expect("could not get window");
    let local_sender = paste_sender.lock().unwrap().clone();

    gloo_events::EventListener::new(&web_window, "paste", move |event| {
        let event = event.dyn_ref::<web_sys::ClipboardEvent>().unwrap_throw();
        if let Some(data) = event.clipboard_data() {
            if let Ok(text) = data.get_data("text") {
                local_sender.send(text.to_owned()).unwrap();
            }
        }
    })
    .forget();
}

fn clipboard(paste_receiver: Res<Mutex<OnPasteReceiver>>, mut wordle_share: ResMut<WordleShare>) {
    if let Ok(text) = paste_receiver.lock().unwrap().try_recv() {
        wordle_share.0 = text;
    }
}
