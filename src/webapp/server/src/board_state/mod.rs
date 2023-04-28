// Code generated by jtd-codegen for Rust v0.2.1

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BoardState {
    #[serde(rename = "over")]
    pub over: bool,

    #[serde(rename = "remaining_tiles")]
    pub remainingTiles: Vec<Tile>,

    #[serde(rename = "table")]
    pub table: Vec<Vec<Option<Box<Player>>>>,

    #[serde(rename = "player_to_move")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playerToMove: Option<Box<Player>>,
}

#[derive(Serialize, Deserialize)]
pub enum Player {
    #[serde(rename = "P1")]
    P1,

    #[serde(rename = "P2")]
    P2,

    #[serde(rename = "P3")]
    P3,

    #[serde(rename = "P4")]
    P4,
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