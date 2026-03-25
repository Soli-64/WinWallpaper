
use rdev::{listen, Key, EventType};
use crate::ui::{
    get_mouse_passthrough_state,
    set_mouse_passthrough_state,
    set_window_alpha,
};

pub fn toggle_window_state(ctx: eframe::egui::Context) {
        
    std::thread::sleep(std::time::Duration::from_millis(1000));
    
    let mut alt_pressed = false;
    
    if let Err(error) = listen(move |event| {
        match event.event_type {
            EventType::KeyPress(key) => {
                match key {
                    Key::Alt | Key::AltGr => {
                        alt_pressed = true;
                    }
                    Key::KeyW => {
                        if alt_pressed {
                            
                            let is_passthrough = get_mouse_passthrough_state();
                            let new_state = !is_passthrough;
                            
                            set_mouse_passthrough_state(new_state);
                            
                            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::MousePassthrough(new_state));
                            
                            if new_state {
                                set_window_alpha(0);
                            } else {
                                set_window_alpha(255);
                            }
                            
                            ctx.request_repaint();
                        }
                    }
                    _ => ()
                }
            }
            EventType::KeyRelease(key) => {
                match key {
                    Key::Alt | Key::AltGr => {
                        alt_pressed = false;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }) {
        println!("Error in hotkey listener: {:?}", error);
    }

}