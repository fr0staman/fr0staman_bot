use teloxide::utils::html::bold;

use crate::{
    config::consts::{TOP_LIMIT, TOP_LIMIT_WITH_CHARTS},
    db::models::{Game, InlineUser},
    enums::Top10Variant,
    lang::{InnerLang, LocaleTag, lng},
};

use super::{flag::Flags, helpers};

pub fn generate_top10_text(
    ltag: LocaleTag,
    top10_info: Vec<InlineUser>,
    chat_type: Top10Variant,
) -> String {
    let summarized = chat_type.summarize();
    let is_win = matches!(summarized, Top10Variant::Win);
    let chat_type = summarized.into_str();

    let text = lng(&format!("InlineTop10Header_{}", chat_type), ltag);

    let header = bold(&text);
    let key = format!("InlineTop10Line_{}", chat_type);

    let mut result = String::with_capacity(512) + &header + "\n";

    for (index, item) in top10_info.iter().enumerate() {
        let value = if is_win { item.win } else { item.weight };

        let code = Flags::from_code(&item.flag).unwrap_or(Flags::Us);
        let flag = code.to_emoji();

        let line = lng(&key, ltag).args(&[
            ("number", (index + 1).to_string()),
            ("flag", flag.to_string()),
            ("name", helpers::escape_links(&item.name)),
            ("value", value.to_string()),
        ]);

        result += &("\n".to_owned() + &line);
    }

    result
}

pub fn generate_chat_top_text(
    ltag: LocaleTag,
    top_info: Vec<Game>,
    offset_multiplier: i64,
    with_chart: bool,
) -> String {
    let top_limit = if with_chart { TOP_LIMIT_WITH_CHARTS } else { TOP_LIMIT };

    let text = lng("GameTopHeader", ltag).args(&[("limit", top_limit)]);
    let header = bold(&text);

    let mut result = String::with_capacity(512) + &header;

    for (index, item) in top_info.iter().enumerate() {
        let value = item.mass;

        let index = (index as i64) + (offset_multiplier * top_limit);

        let line = lng("GameTopLine", ltag).args(&[
            ("number", (index + 1).to_string()),
            ("name", helpers::escape_links(&item.name)),
            ("value", value.to_string()),
        ]);

        result += &("\n".to_owned() + &line);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{game, init_lang, inline_user};

    fn hand_pig(name: &str, weight: i32, win: i32, flag: &str) -> InlineUser {
        let mut pig = inline_user(1, 1, weight);
        pig.name = name.to_owned();
        pig.win = win;
        pig.flag = flag.to_owned();
        pig
    }

    fn chat_pig(name: &str, mass: i32) -> Game {
        let mut pig = game(mass);
        pig.name = name.to_owned();
        pig
    }


    #[test]
    fn an_empty_top10_is_just_the_header() {
        let ltag = init_lang();
        let text = generate_top10_text(ltag, vec![], Top10Variant::Global);

        assert!(!text.is_empty());
        assert!(!text.contains("lang:"), "{text}");
        assert!(text.contains("<b>"), "the header is bolded: {text}");
    }

    #[test]
    fn top10_lines_are_numbered_from_one() {
        let ltag = init_lang();
        let pigs = vec![
            hand_pig("First", 300, 5, "uk"),
            hand_pig("Second", 200, 3, "en"),
            hand_pig("Third", 100, 1, "az"),
        ];

        let text = generate_top10_text(ltag, pigs, Top10Variant::Global);

        assert!(text.contains("First"));
        assert!(text.contains("Second"));
        assert!(text.contains("Third"));
        let first = text.find("First").unwrap();
        let second = text.find("Second").unwrap();
        let third = text.find("Third").unwrap();
        assert!(first < second && second < third, "{text}");
    }

    #[test]
    fn the_win_board_shows_wins_and_the_others_show_weight() {
        let ltag = init_lang();
        let pigs = vec![hand_pig("Champ", 4242, 77, "uk")];

        let weight_board =
            generate_top10_text(ltag, pigs.clone_for_test(), Top10Variant::Global);
        assert!(weight_board.contains("4242"), "{weight_board}");

        let win_board = generate_top10_text(ltag, pigs, Top10Variant::Win);
        assert!(win_board.contains("77"), "{win_board}");
    }

    #[test]
    fn the_private_variants_render_as_their_public_form() {
        let ltag = init_lang();

        let public = generate_top10_text(ltag, vec![], Top10Variant::Global);
        let private = generate_top10_text(ltag, vec![], Top10Variant::PGlobal);

        assert_eq!(public, private);
    }

    #[test]
    fn an_unknown_flag_falls_back_rather_than_panicking() {
        let ltag = init_lang();
        let pigs = vec![hand_pig("Mystery", 10, 0, "definitely-not-a-code")];

        let text = generate_top10_text(ltag, pigs, Top10Variant::Global);

        assert!(text.contains("Mystery"), "{text}");
        assert!(text.contains(Flags::Us.to_emoji()), "{text}");
    }

    #[test]
    fn links_in_a_pig_name_are_stripped() {
        let ltag = init_lang();
        let pigs = vec![hand_pig("spam https://t.me/evil", 10, 0, "uk")];

        let text = generate_top10_text(ltag, pigs, Top10Variant::Global);

        assert!(!text.contains("https://"), "{text}");
        assert!(!text.contains("t.me"), "{text}");
    }


    #[test]
    fn an_empty_chat_top_is_just_the_header() {
        let ltag = init_lang();
        let text = generate_chat_top_text(ltag, vec![], 0, false);

        assert!(!text.is_empty());
        assert!(!text.contains("lang:"), "{text}");
        assert!(text.contains("<b>"), "the header is bolded: {text}");
    }

    #[test]
    fn chat_top_numbering_continues_across_pages() {
        let ltag = init_lang();
        let pigs = vec![chat_pig("A", 30), chat_pig("B", 20)];

        let first_page = generate_chat_top_text(ltag, pigs_for(&pigs), 0, false);
        assert!(first_page.contains('1'), "{first_page}");
        let second_page = generate_chat_top_text(ltag, pigs_for(&pigs), 1, false);
        assert!(
            second_page.contains(&(TOP_LIMIT + 1).to_string()),
            "{second_page}"
        );
    }

    #[test]
    fn the_header_reports_the_page_size_in_use() {
        let ltag = init_lang();

        let plain = generate_chat_top_text(ltag, vec![], 0, false);
        let charted = generate_chat_top_text(ltag, vec![], 0, true);

        assert!(plain.contains(&TOP_LIMIT.to_string()), "{plain}");
        assert!(
            charted.contains(&TOP_LIMIT_WITH_CHARTS.to_string()),
            "{charted}"
        );
    }

    #[test]
    fn chat_top_offsets_use_the_chart_page_size_when_charts_are_on() {
        let ltag = init_lang();
        let pigs = vec![chat_pig("A", 30)];

        let text = generate_chat_top_text(ltag, pigs, 1, true);

        assert!(
            text.contains(&(TOP_LIMIT_WITH_CHARTS + 1).to_string()),
            "{text}"
        );
    }

    #[test]
    fn links_in_a_chat_pig_name_are_stripped() {
        let ltag = init_lang();
        let pigs = vec![chat_pig("join t.me/evil now", 10)];

        let text = generate_chat_top_text(ltag, pigs, 0, false);

        assert!(!text.contains("t.me"), "{text}");
    }

    // Small helpers so the tests above can reuse fixtures without `Clone`
    // on the diesel models.
    trait CloneForTest {
        fn clone_for_test(&self) -> Self;
    }

    impl CloneForTest for Vec<InlineUser> {
        fn clone_for_test(&self) -> Self {
            self.iter()
                .map(|p| hand_pig(&p.name, p.weight, p.win, &p.flag))
                .collect()
        }
    }

    fn pigs_for(pigs: &[Game]) -> Vec<Game> {
        pigs.iter().map(|p| chat_pig(&p.name, p.mass)).collect()
    }
}
