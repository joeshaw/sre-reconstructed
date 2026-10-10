// The game modules' own static variables.

pub struct ModState {
    pub start: i32,                                       // play.cpp g_start
    pub warned: i16,                                      // play.cpp time warning
    pub local: i16,                                       // play.cpp local-play flags
    pub idle_last: Option<i32>,                           // play.cpp idle hook
    pub nextevent: i32,                                   // play.cpp event timer
    pub researchcount: i16,                               // research.cpp cached count
    pub chosen: i16,                                      // editors.cpp list selection
    pub message: Option<Box<crate::messages::Message>>,   // messages.cpp TheMessage
}

impl Default for ModState {
    fn default() -> Self {
        ModState {
            start: 0,
            warned: 0,
            local: 0,
            idle_last: None,
            nextevent: 0,
            researchcount: -1,
            chosen: 1,
            message: None,
        }
    }
}
