mod app;
mod assets;
mod camera;
#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod leaderboard;
mod menu;
mod util;

fn main() {
    app::start();
}
