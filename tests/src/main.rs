#[derive(Debug)]
struct PlayerData {
    player_id: i32,
    player_name: String,
}

#[either_field::make_template(
    ScoreWithPlayer:
    [
        player: PlayerData
    ],
    ScoreWithoutPlayer:
    [
        player_name: String
    ]
)]
#[derive(Debug)]
struct Score<A> {
    player_name: either_field::either!(() | String),
    player: either_field::either!(i32 | PlayerData),
    value: A,
}

fn main() {
    let x = ScoreWithPlayer {
        player_name: (),
        player: PlayerData {
            player_id: 1,
            player_name: String::from("Example"),
        },
        value: 0,
    };
    let y = ScoreWithoutPlayer {
        player_name: String::from("Example"),
        player: 1,
        value: 0,
    };
    println!("{x:#?}");
    println!("{y:#?}");
}