use serde::Serialize;

const NARUTO_R34_BLOCK: &[&str] = &[
    "webm", "comic", "sketch", "dialogue", "video", "6+girls", "small_breasts", "3girls", "6girls", "4girls",
    "5girls", "7girls", "grimotk", "wanaata", "georugu13", "kristiadder", "prinnydood",
    "atelier_gons", "midiman", "amputee", "amputation", "quadruple_amputee", "dismemberment", "anthro", "kushishekku",
    "6+futas", "6+boys", "bookman_v", "zoonaru", "3d", "speech_bubble", "monochrome",
    "male_only", "gore", "guro", "necrophilia", "blood", "extreme_content", "bluethebone", "10girls",
    "wooden_horse", "stationary_restraints", "male_only", "gore", "guro", "necrophilia", "blood", "extreme_content", "bluethebone",
    "10girls", "wooden_horse", "stationary_restraints", "diphallia", "diphallism", "monophallia",
    "what", "detachable", "detachable_head", "execution", "decapitation", "death", "rope",
    "gagged", "gag"
];

const BLEACH_R34_BLOCK: &[&str] = &[
    "webm", "comic", "sketch", "dialogue", "video", "kagami", "lillie_(pokemon)", "lana_(pokemon)", "6+girls", "small_breasts",
    "3girls", "6girls", "4girls", "5girls", "7girls", "age_difference", "grimotk", "wanaata",
    "georugu13", "kristiadder", "prinnydood", "atelier_gons", "midiman", "amputee",
    "amputation", "quadruple_amputee", "dismemberment", "furry", "anthro", "kushishekku",
    "6+futas", "6+boys", "bookman_v", "3d", "speech_bubble", "monochrome",
    "male_only", "gore", "guro", "necrophilia", "blood", "extreme_content", "bluethebone", "10girls",
    "wooden_horse", "stationary_restraints", "male_only", "gore", "guro", "necrophilia", "blood", "extreme_content", "bluethebone",
    "10girls", "wooden_horse", "stationary_restraints", "diphallia", "diphallism", "monophallia",
    "what", "detachable", "detachable_head", "execution", "decapitation", "death", "rope",
    "gagged", "gag"
];

const GLOBAL_R34_BLOCK: &[&str] = &[
    "webm", "comic", "sketch", "dialogue", "video", "lillie_(pokemon)", "lana_(pokemon)", "6+girls", "small_breasts",
    "flat_chest", "3girls", "6girls", "4girls", "5girls", "7girls", "grimotk", "wanaata",
    "georugu13", "kristiadder", "prinnydood", "atelier_gons", "midiman", "amputee",
    "amputation", "quadruple_amputee", "dismemberment", "anthro", "garakuta_(garakuta_no_gomibako)",
    "kushishekku", "6+futas", "6+boys", "bookman_v", "3d", "speech_bubble", "monochrome",
    "male_only", "gore", "guro", "necrophilia", "blood", "extreme_content", "bluethebone",
    "10girls", "wooden_horse", "stationary_restraints", "diphallia", "diphallism", "monophallia",
    "what", "detachable", "detachable_head", "execution", "decapitation", "death", "rope",
    "gagged", "gag", "pela_(honkai:_star_rail)", "lynx_(honkai:_star_rail)", "hook_(honkai:_star_rail)",
    "bailu_(honkai:_star_rail)", "pom-pom_(honkai:_star_rail)"
];

const GLOBAL_DANBOORU_BLOCK: &[&str] = &[
    "lolicon", "shotacon", "comic", "video", "animated", "multiple_girls", "small_breasts",
    "flat_chest", "paimon_(genshin_impact)", "3d", "speech_bubble", "speech_bubble", "monochrome", "male_only", "bluethebone", "10girls"
];

const AI_R34_BLOCK: &[&str] = &[
    "webm", "comic", "sketch", "dialogue", "video", "lillie_(pokemon)", "lana_(pokemon)", "6+girls", "small_breasts",
    "3girls", "6girls", "4girls", "5girls", "7girls", "grimotk", "wanaata",
    "georugu13", "kristiadder", "prinnydood", "atelier_gons", "midiman", "amputee",
    "amputation", "quadruple_amputee", "dismemberment", "anthro", "garakuta_(garakuta_no_gomibako)",
    "kushishekku", "6+futas", "6+boys", "bookman_v", "male_only", "speech_bubble", "monochrome",
    "gore", "guro", "necrophilia", "blood", "extreme_content", "male_only", "gore", "guro", "necrophilia", "blood", "extreme_content", "bluethebone",
    "10girls", "wooden_horse", "stationary_restraints", "diphallia", "diphallism", "monophallia",
    "what", "detachable", "detachable_head", "execution", "decapitation", "death", "rope",
    "gagged", "gag"
];

const FURRY_BLOCK: &[&str] = &[
    "webm", "comic", "sketch", "dialogue", "video", "lillie_(pokemon)", "lana_(pokemon)", "6+girls", "small_breasts",
    "flat_chest", "3girls", "6girls", "4girls", "5girls", "7girls", "grimotk", "wanaata",
    "georugu13", "kristiadder", "prinnydood", "atelier_gons", "midiman", "amputee",
    "amputation", "quadruple_amputee", "dismemberment", "anthro", "garakuta_(garakuta_no_gomibako)",
    "kushishekku", "6+futas", "6+boys", "bookman_v", "3d", "speech_bubble", "monochrome",
    "male_only", "gore", "guro", "necrophilia", "blood", "extreme_content", "bluethebone", "10girls", "wooden_horse", "stationary_restraints",
    "age_difference", "cub", "young", "younger_female", "smaller_female", "male_only", "gore", "guro", "necrophilia", "blood", "extreme_content", "bluethebone",
    "10girls", "wooden_horse", "stationary_restraints", "diphallia", "diphallism", "monophallia",
    "what", "detachable", "detachable_head", "execution", "decapitation", "death", "rope",
    "gagged", "gag"
];

#[derive(Serialize)]
pub struct BlocklistEntry {
    pub name: &'static str,
    pub tags: &'static [&'static str],
}

/// Single registry of all blocklists with their legacy export name names.
pub const BLOCKLISTS: &[BlocklistEntry] = &[
    BlocklistEntry { name: "narutoR34Block", tags: NARUTO_R34_BLOCK },
    BlocklistEntry { name: "bleachR34Block", tags: BLEACH_R34_BLOCK },
    BlocklistEntry { name: "globalR34Block", tags: GLOBAL_R34_BLOCK },
    BlocklistEntry { name: "globalDanbooruBlock", tags: GLOBAL_DANBOORU_BLOCK },
    BlocklistEntry { name: "aiR34Block", tags: AI_R34_BLOCK },
    BlocklistEntry { name: "furryBlock", tags: FURRY_BLOCK },
];
