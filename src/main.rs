use xilem::{
    view::{flex, Axis},
    WidgetView, Xilem,
};

mod disable;
mod game;
mod tile;

use disable::disable_if;
use tile::{tile, Tile};

struct Ultimate {
    tiles: [Option<Player>; 81],
    minisquares: [Option<Player>; 9],
    whose_turn: Option<Player>, // None if ended
    local_player: Player,
    history: Vec<usize>, // coord
}

impl Ultimate {
    // just here to shrink the syntax in app() lol
    fn tile(&self, coord: usize) -> Tile {
        tile(coord, self.tiles[coord], self.is_playable(coord))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Player {
    Nought,
    Cross,
}

fn main() {
    let ev_builder = xilem::EventLoop::with_user_event();
    let ult = Ultimate {
        tiles: [None; 81],
        minisquares: [None; 9],
        whose_turn: Some(Player::Cross),
        local_player: Player::Cross,
        history: Vec::new(),
    };
    Xilem::new(ult, app)
        .run_windowed(ev_builder, "Ultimate 3".to_owned())
        .unwrap();
}

fn app(ult: &mut Ultimate) -> impl WidgetView<Ultimate> {
    let minisquare = |topleft: usize| {
        let row = |i| {
            flex((ult.tile(i), ult.tile(i + 1), ult.tile(i + 2)))
                .gap(2.)
                .direction(Axis::Horizontal)
        };
        flex((row(topleft), row(topleft + 9), row(topleft + 18))).gap(2.)
    };
    let row = |i| {
        flex((minisquare(i), minisquare(i + 3), minisquare(i + 6)))
            .gap(4.)
            .direction(Axis::Horizontal)
    };
    disable_if(
        ult.whose_turn != Some(ult.local_player),
        flex((row(0), row(27), row(54)))
            .gap(4.)
            .main_axis_alignment(xilem::view::MainAxisAlignment::Center),
    )
}
