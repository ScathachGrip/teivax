pub mod nikke;
pub mod arknights;

#[derive(Debug, Clone)]
pub struct Anime {
    pub id: &'static str,
    pub title: &'static str,
    pub provider: &'static str,
    pub tags: &'static [&'static str],
}

pub const REGISTRY: &[Anime] = &[
    Anime {
        id: "nikke",
        title: "Nikke",
        provider: "rule34",
        tags: nikke::TAGS,
    },
    Anime {
        id: "arknights",
        title: "Arknights",
        provider: "rule34",
        tags: arknights::TAGS,
    },
];

pub fn by_id(id: &str) -> Option<&'static Anime> {
    REGISTRY.iter().find(|a| a.id == id)
}
