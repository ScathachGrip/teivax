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
pub mod data_gif;
pub mod data_gif_nsfw;
pub mod hentai_yandere;
pub mod ai_sex;
pub mod gif_sex;
pub mod blocklists;
pub mod global_anime_girls;

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct GifEntry {
    pub tags: &'static str,
    pub image: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub enum TagData {
    Flat(&'static [&'static str]),
    Gif(&'static [GifEntry]),
}

impl serde::Serialize for TagData {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TagData::Flat(arr) => arr.serialize(serializer),
            TagData::Gif(arr) => arr.serialize(serializer),
        }
    }
}

impl TagData {
    pub fn len(&self) -> usize {
        match self {
            TagData::Flat(arr) => arr.len(),
            TagData::Gif(arr) => arr.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Anime {
    pub id: &'static str,
    pub title: &'static str,
    pub provider: &'static str,
    pub tags: TagData,
}

pub use blocklists::BLOCKLISTS;
pub use global_anime_girls::GIRLS as GLOBAL_ANIME_GIRLS;

pub const REGISTRY: &[Anime] = &[
    Anime { id: "nikke", title: "Nikke", provider: "rule34", tags: TagData::Flat(nikke::TAGS) },
    Anime { id: "arknights", title: "Arknights", provider: "rule34", tags: TagData::Flat(arknights::TAGS) },
    Anime { id: "bluearchive", title: "Blue Archive", provider: "rule34", tags: TagData::Flat(bluearchive::TAGS) },
    Anime { id: "azurlane", title: "Azur Lane", provider: "rule34", tags: TagData::Flat(azurlane::TAGS) },
    Anime { id: "fgo", title: "Fate/Grand Order", provider: "rule34", tags: TagData::Flat(fgo::TAGS) },
    Anime { id: "genshin", title: "Genshin Impact", provider: "rule34", tags: TagData::Flat(genshin::TAGS) },
    Anime { id: "genshin_danbooru", title: "Genshin Impact (Danbooru)", provider: "danbooru", tags: TagData::Flat(genshin_danbooru::TAGS) },
    Anime { id: "honkai_starrail", title: "Honkai: Star Rail", provider: "rule34", tags: TagData::Flat(honkai_starrail::TAGS) },
    Anime { id: "girls_frontline", title: "Girls' Frontline", provider: "rule34", tags: TagData::Flat(girls_frontline::TAGS) },
    Anime { id: "naruto", title: "Naruto", provider: "rule34", tags: TagData::Flat(naruto::TAGS) },
    Anime { id: "bleach", title: "Bleach", provider: "rule34", tags: TagData::Flat(bleach::TAGS) },
    Anime { id: "vtubers", title: "VTubers", provider: "rule34", tags: TagData::Flat(vtubers::TAGS) },
    Anime { id: "danbooru_sex", title: "Danbooru Sex Tags", provider: "danbooru", tags: TagData::Flat(danbooru_sex::TAGS) },
    Anime { id: "data_gif", title: "GIFs", provider: "others", tags: TagData::Gif(data_gif::DATAGIF) },
    Anime { id: "data_gif_nsfw", title: "NSFW GIFs", provider: "others", tags: TagData::Gif(data_gif_nsfw::DATAGIFNSFW) },
    Anime { id: "hentai_yandere", title: "Hentai Yandere Tags", provider: "yandere", tags: TagData::Flat(hentai_yandere::TAGS) },
    Anime { id: "ai_sex", title: "AI Sex Tags", provider: "others", tags: TagData::Flat(ai_sex::TAGS) },
    Anime { id: "gif_sex", title: "GIF Sex Tags", provider: "others", tags: TagData::Flat(gif_sex::TAGS) },
];

pub fn by_id(id: &str) -> Option<&'static Anime> {
    REGISTRY.iter().find(|a| a.id == id)
}
