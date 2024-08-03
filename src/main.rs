use std::sync::Mutex;

use futures::future::Either;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};
use xilem::{
    core::{adapt, fork, MessageProxy},
    view::{async_repeat, async_repeat_raw, button, flex, label, sized_box, textbox, Axis},
    WidgetView, Xilem,
};

mod disable;
mod game;
mod tile;

use disable::disable_if;
use tile::{tile, Tile};

enum AppState {
    MainMenu(MainMenu),
    WaitingForOpponent,
    Connecting(String),
    InGame(Ultimate),
}

impl AppState {
    fn expect_main_menu_mut(&mut self) -> &mut MainMenu {
        match self {
            AppState::MainMenu(menu) => menu,
            _ => panic!("expected main menu but app was in another state!"),
        }
    }

    fn expect_game_mut(&mut self) -> &mut Ultimate {
        match self {
            AppState::InGame(ult) => ult,
            _ => panic!("expected in-game but app was in another state!"),
        }
    }
}

struct MainMenu {
    remote_address: String,
}

struct Ultimate {
    tiles: [Option<Player>; 81],
    minisquares: [Option<Player>; 9],
    whose_turn: Option<Player>, // None if ended
    local_player: Player,
    history: Vec<usize>, // coord

    send: Option<Sender<usize>>,
    recv: Option<tokio::net::tcp::OwnedReadHalf>,

    for_recv_task: Option<(Receiver<usize>, tokio::net::tcp::OwnedWriteHalf)>,
}

impl Ultimate {
    fn local_multiplayer() -> Self {
        Ultimate {
            tiles: [None; 81],
            minisquares: [None; 9],
            whose_turn: Some(Player::Cross),
            local_player: Player::Cross,
            history: Vec::new(),

            send: None,
            recv: None,

            for_recv_task: None,
        }
    }

    fn network_multiplayer(stream: TcpStream, local_player: Player) -> Self {
        let (net_rx, net_tx) = stream.into_split();
        let (ui_tx, task_rx) = tokio::sync::mpsc::channel::<usize>(1);
        Ultimate {
            tiles: [None; 81],
            minisquares: [None; 9],
            whose_turn: Some(Player::Cross),
            local_player,
            history: Vec::new(),

            recv: Some(net_rx),
            send: Some(ui_tx),

            for_recv_task: Some((task_rx, net_tx)),
        }
    }

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

impl std::ops::Not for Player {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Player::Nought => Player::Cross,
            Player::Cross => Player::Nought,
        }
    }
}

fn main() {
    let ev_builder = xilem::EventLoop::with_user_event();
    let app_state = AppState::MainMenu(MainMenu {
        remote_address: String::new(),
    });
    Xilem::new(app_state, app)
        .run_windowed(ev_builder, "Ultimate 3".to_owned())
        .unwrap();
}

fn app(s: &mut AppState) -> impl WidgetView<AppState> {
    match s {
        AppState::MainMenu(menu_state) => menu(menu_state).boxed(),
        AppState::WaitingForOpponent => fork(
            label("Waiting for opponent..."),
            async_repeat(listen_for_opponent, |s, stream| {
                *s = AppState::InGame(Ultimate::network_multiplayer(stream, Player::Cross));
            }),
        )
        .boxed(),
        AppState::Connecting(address) => {
            let address = address.clone();
            fork(
                label("Connecting to opponent..."),
                async_repeat_raw(
                    move |proxy| connect_to_opponent(proxy, address.clone()),
                    |s, stream| {
                        *s =
                            AppState::InGame(Ultimate::network_multiplayer(stream, Player::Nought));
                    },
                ),
            )
        }
        .boxed(),
        AppState::InGame(ult) => {
            // eeewwwwww
            let net_rx = Mutex::new(ult.recv.take());
            let listen_for_move = move |proxy: MessageProxy<u8>| {
                if let Some(mut net_rx) = net_rx.lock().unwrap().take() {
                    Either::Left(async move {
                        while let Ok(coord) = net_rx.read_u8().await {
                            let _ = proxy.message(coord);
                        }
                    })
                } else {
                    Either::Right(std::future::ready(()))
                }
            };
            let on_receive_move = |s: &mut Ultimate, coord| {
                s.handle_move(!s.local_player, coord as usize);
            };
            adapt(
                fork(
                    game(ult),
                    async_repeat_raw(listen_for_move, on_receive_move),
                ),
                |s: &mut AppState, thunk| thunk.call(s.expect_game_mut()),
            )
            .boxed()
        }
    }
}

async fn listen_for_opponent(proxy: MessageProxy<TcpStream>) {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:25567")
        .await
        .unwrap();
    let (stream, remote_addr) = listener.accept().await.unwrap();
    tracing::info!(?remote_addr, "connected!");
    let _ = proxy.message(stream);
}

async fn connect_to_opponent(proxy: MessageProxy<TcpStream>, remote_addr: String) {
    let stream = tokio::net::TcpStream::connect(&remote_addr).await.unwrap();
    tracing::info!(remote_addr, "connected!");
    let _ = proxy.message(stream);
}

// The state types differ because we know the state is MainMenu now and all the interesting fields
// are in there, but we need to be able to change it later
fn menu(s: &mut MainMenu) -> impl WidgetView<AppState> {
    let connect_to_game_ui = flex((
        sized_box(textbox(
            s.remote_address.clone(),
            |s: &mut AppState, text| s.expect_main_menu_mut().remote_address = text,
        ))
        .width(160.),
        button("Connect to game", |s: &mut AppState| {
            *s = AppState::Connecting(s.expect_main_menu_mut().remote_address.clone());
        }),
    ))
    .direction(Axis::Horizontal);
    flex((
        connect_to_game_ui,
        button("Host game", |s| {
            *s = AppState::WaitingForOpponent;
        }),
        button("Start local game", |s| {
            *s = AppState::InGame(Ultimate::local_multiplayer());
        }),
    ))
    .main_axis_alignment(xilem::view::MainAxisAlignment::Center)
}

fn game(ult: &mut Ultimate) -> impl WidgetView<Ultimate> {
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
    let ui = flex((row(0), row(27), row(54)))
        .gap(4.)
        .main_axis_alignment(xilem::view::MainAxisAlignment::Center);
    let for_recv_task = Mutex::new(ult.for_recv_task.take());
    fork(
        ui,
        async_repeat_raw(
            move |_proxy| {
                if let Some((mut task_rx, mut net_tx)) = for_recv_task.lock().unwrap().take() {
                    Either::Left(async move {
                        while let Some(coord) = task_rx.recv().await {
                            let _ = net_tx
                                .write_all(&[coord as u8])
                                .await
                                .inspect_err(|e| tracing::error!(?e));
                        }
                    })
                } else {
                    Either::Right(std::future::ready(()))
                }
            },
            |_s, _msg: ()| unreachable!("the future does not send any messages"),
        ),
    )
}
