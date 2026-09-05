pub mod cmd;
pub mod downloader {
    pub mod play;
    pub mod reels;
}
pub mod general {
    pub mod info;
    pub mod menu;
    pub mod ping;
}
pub mod group {
    pub mod delete;
    pub mod demote;
    pub mod gc;
    pub mod hidetag;
    pub mod kick;
    pub mod mute;
    pub mod promote;
    pub mod setephemeral;
}
pub mod root {
    pub mod cache;
    pub mod exec;
    pub mod set;
    pub mod spamedit;
}
pub mod tools {
    pub mod block;
    pub mod debug;
    pub mod listblock;
    pub mod rvo;
    pub mod unblock;
}
