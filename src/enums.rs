#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Page {
    Home,
    Launch,
    AddGame,
    ProcTime,
    Settings,
}
