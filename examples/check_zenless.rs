#[path = "../src/data/zenless_zone_zero.rs"]
mod target_data;

#[path = "../src/check_util.rs"]
mod check_util;

const TAGS: &[&str] = target_data::TAGS;

fn main() {
    check_util::check_danbooru(TAGS, "zenless");
}
