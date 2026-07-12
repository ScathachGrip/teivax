pub mod nikke;
pub mod arknights;
pub mod bluearchive;
pub mod azurlane;
pub mod fgo;
pub mod genshin;
pub mod genshin_danbooru;
pub mod honkai_starrail;
pub mod girls_frontline;
pub mod naruto;
pub mod bleach;
pub mod vtubers;
pub mod danbooru_sex;
pub mod gif_sex;
pub mod hentai_yandere;
pub mod ai_sex;
pub mod blocklists;
pub mod global_anime_girls;

#[derive(Debug, Clone)]
pub struct Anime {
    pub id: &'static str,
    pub title: &'static str,
    pub provider: &'static str,
    pub tags: &'static [&'static str],
}

pub use blocklists::BLOCKLISTS;
pub use global_anime_girls::GIRLS as GLOBAL_ANIME_GIRLS;

pub const REGISTRY: &[Anime] = &[
    Anime { id: "nikke", title: "Nikke", provider: "rule34", tags: nikke::TAGS },
    Anime { id: "arknights", title: "Arknights", provider: "rule34", tags: arknights::TAGS },
    Anime { id: "bluearchive", title: "Blue Archive", provider: "rule34", tags: bluearchive::TAGS },
    Anime { id: "azurlane", title: "Azur Lane", provider: "rule34", tags: azurlane::TAGS },
    Anime { id: "fgo", title: "Fate/Grand Order", provider: "rule34", tags: fgo::TAGS },
    Anime { id: "genshin", title: "Genshin Impact", provider: "rule34", tags: genshin::TAGS },
    Anime { id: "genshin_danbooru", title: "Genshin Impact (Danbooru)", provider: "danbooru", tags: genshin_danbooru::TAGS },
    Anime { id: "honkai_starrail", title: "Honkai: Star Rail", provider: "rule34", tags: honkai_starrail::TAGS },
    Anime { id: "girls_frontline", title: "Girls' Frontline", provider: "rule34", tags: girls_frontline::TAGS },
    Anime { id: "naruto", title: "Naruto", provider: "rule34", tags: naruto::TAGS },
    Anime { id: "bleach", title: "Bleach", provider: "rule34", tags: bleach::TAGS },
    Anime { id: "vtubers", title: "VTubers", provider: "rule34", tags: vtubers::TAGS },
    Anime { id: "danbooru_sex", title: "Danbooru Sex Tags", provider: "danbooru", tags: danbooru_sex::TAGS },
    Anime { id: "gif_sex", title: "GIF Sex Tags", provider: "others", tags: gif_sex::TAGS },
    Anime { id: "hentai_yandere", title: "Hentai Yandere Tags", provider: "yandere", tags: hentai_yandere::TAGS },
    Anime { id: "ai_sex", title: "AI Sex Tags", provider: "others", tags: ai_sex::TAGS },
];

pub fn by_id(id: &str) -> Option<&'static Anime> {
    REGISTRY.iter().find(|a| a.id == id)
}
