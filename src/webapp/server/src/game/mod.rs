// Code generated by jtd-codegen for Rust v0.2.1

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GamePlayer {
    #[serde(rename = "color")]
    pub color: Color,

    #[serde(rename = "score")]
    pub score: i8,

    #[serde(rename = "user")]
    pub user: User,
}

#[derive(Serialize, Deserialize)]
pub enum GameStatus {
    #[serde(rename = "ENDED")]
    Ended,

    #[serde(rename = "ONGOING")]
    Ongoing,

    #[serde(rename = "WAITING_FOR_PLAYERS")]
    WaitingForPlayers,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "players")]
    pub players: Vec<GamePlayer>,

    #[serde(rename = "status")]
    pub status: GameStatus,

    #[serde(rename = "board")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board: Option<Box<Board>>,
}

#[derive(Serialize, Deserialize)]
pub struct BoardRemainingTiles {
    #[serde(rename = "color")]
    pub color: Color,

    #[serde(rename = "tiles")]
    pub tiles: Vec<Tile>,
}

#[derive(Serialize, Deserialize)]
pub struct BoardScore {
    #[serde(rename = "color")]
    pub color: Color,

    #[serde(rename = "score")]
    pub score: i8,
}

#[derive(Serialize, Deserialize)]
pub struct Board {
    #[serde(rename = "over")]
    pub over: bool,

    #[serde(rename = "remaining_tiles")]
    pub remainingTiles: BoardRemainingTiles,

    #[serde(rename = "score")]
    pub score: BoardScore,

    #[serde(rename = "table")]
    pub table: Vec<Vec<Option<Box<Color>>>>,

    #[serde(rename = "color_to_move")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colorToMove: Option<Box<Color>>,
}

#[derive(Serialize, Deserialize)]
pub enum Color {
    #[serde(rename = "C1")]
    C1,

    #[serde(rename = "C2")]
    C2,

    #[serde(rename = "C3")]
    C3,

    #[serde(rename = "C4")]
    C4,
}

#[derive(Serialize, Deserialize)]
pub enum Tile {
    #[serde(rename = "T1")]
    T1,

    #[serde(rename = "T2")]
    T2,

    #[serde(rename = "T3")]
    T3,

    #[serde(rename = "T4")]
    T4,

    #[serde(rename = "T5")]
    T5,

    #[serde(rename = "T6")]
    T6,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: String,
}
