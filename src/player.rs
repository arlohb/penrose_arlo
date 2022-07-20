use mpris::{Player, PlayerFinder};

thread_local! {
    static PLAYER_FINDER: PlayerFinder = PlayerFinder::new().unwrap();
}

pub fn with_player<R>(f: fn(Player) -> Option<R>) -> Option<R> {
    PLAYER_FINDER.with(|finder| {
        if let Ok(player) = finder.find_active() {
            f(player)
        } else {
            None
        }
    })
}
