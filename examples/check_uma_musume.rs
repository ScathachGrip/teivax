#[path = "../src/data/uma_musume.rs"]
mod target_data;

#[path = "../src/check_util.rs"]
mod check_util;

const TAGS: &[&str] = target_data::TAGS;

fn main() {
    check_util::check_rule34(TAGS, "uma_musume");
}
