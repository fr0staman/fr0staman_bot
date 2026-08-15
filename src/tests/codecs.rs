//! The hand-rolled string codecs. Their output lives in chat history
//! forever, so an old payload must still decode today.

use std::str::FromStr;

use crate::enums::{InlineResults, Top10Variant};


fn all_variants() -> Vec<InlineResults> {
    vec![
        InlineResults::GetStartDuel,
        InlineResults::GetTop10Info,
        InlineResults::GetHryakInfo,
        InlineResults::GetMoreInfo,
        InlineResults::NameHryakInfo,
        InlineResults::RenameHryakInfo,
        InlineResults::DayPigInfo,
        InlineResults::FlagInfo,
        InlineResults::FlagEmptyInfo,
        InlineResults::FlagChangeInfo("ua".to_owned()),
        InlineResults::LangInfo,
        InlineResults::LangEmptyInfo,
        InlineResults::LangChangeInfo("uk".to_owned()),
        InlineResults::CpuOcInfo,
        InlineResults::RamOcInfo,
        InlineResults::GpuOcInfo,
        InlineResults::HruVoice(42),
        InlineResults::PigGif(-7),
        InlineResults::ErrorInfo,
        InlineResults::ErrorParse,
        InlineResults::NoResults,
    ]
}

#[test]
fn every_inline_result_round_trips() {
    for variant in all_variants() {
        let encoded = variant.to_string_with_args();
        let decoded = InlineResults::from_str_with_args(&encoded)
            .unwrap_or_else(|| panic!("{encoded} failed to decode"));

        assert_eq!(decoded, variant, "encoded as {encoded}");
    }
}

#[test]
fn the_encoding_always_carries_the_delimiter() {
    // `from_str_with_args` starts with `split_once('|')`, so a payload-free
    // variant must still emit a trailing `|` or it will not decode at all.
    for variant in all_variants() {
        let encoded = variant.to_string_with_args();
        assert!(
            encoded.contains(InlineResults::DELIMITER),
            "{encoded} has no delimiter"
        );
    }
}

#[test]
fn payload_free_variants_encode_with_an_empty_tail() {
    assert_eq!(InlineResults::GetStartDuel.to_string_with_args(), "get_start_duel|");
    assert_eq!(InlineResults::NoResults.to_string_with_args(), "no_results|");
}

#[test]
fn payload_carrying_variants_encode_their_value() {
    assert_eq!(
        InlineResults::FlagChangeInfo("gb".to_owned()).to_string_with_args(),
        "flag_change_info|gb"
    );
    assert_eq!(InlineResults::HruVoice(11).to_string_with_args(), "hru_voice|11");
    assert_eq!(InlineResults::PigGif(-1).to_string_with_args(), "pig_gif|-1");
}

#[test]
fn a_result_id_without_a_delimiter_is_rejected() {
    // Nothing produces these today, but a hand-written or pre-delimiter
    // `result_id` arriving from an old chat would silently drop.
    assert_eq!(InlineResults::from_str_with_args("day_pig_info"), None);
    assert_eq!(InlineResults::from_str_with_args(""), None);
}

#[test]
fn an_unknown_variant_name_is_rejected() {
    assert_eq!(InlineResults::from_str_with_args("nope|"), None);
    assert_eq!(InlineResults::from_str_with_args("GetStartDuel|"), None);
}

#[test]
fn an_empty_payload_decodes_to_the_default_value() {
    // Pins strum's behaviour: `from_str` builds the variant with
    // `Default::default()`, and the code then overwrites it with the parsed
    // tail. An empty tail must therefore survive for String payloads...
    assert_eq!(
        InlineResults::from_str_with_args("flag_change_info|"),
        Some(InlineResults::FlagChangeInfo(String::new()))
    );

    // ...and fail for numeric ones, since "" is not an i16.
    assert_eq!(InlineResults::from_str_with_args("hru_voice|"), None);
}

#[test]
fn a_payload_containing_the_delimiter_survives() {
    // `split_once` takes the *first* delimiter, so the remainder is kept
    // whole.
    assert_eq!(
        InlineResults::from_str_with_args("flag_change_info|a|b"),
        Some(InlineResults::FlagChangeInfo("a|b".to_owned()))
    );
}

#[test]
fn an_out_of_range_numeric_payload_is_rejected_rather_than_wrapping() {
    // `inline_voices.id` / `inline_gifs.id` are i16 in the schema.
    assert!(InlineResults::from_str_with_args("hru_voice|32767").is_some());
    assert_eq!(InlineResults::from_str_with_args("hru_voice|32768"), None);
    assert_eq!(InlineResults::from_str_with_args("pig_gif|-32769"), None);
    assert_eq!(InlineResults::from_str_with_args("pig_gif|abc"), None);
}

#[test]
fn the_wire_names_are_stable() {
    // These strings sit in inline results the bot has already sent; renaming
    // a variant would orphan them.
    let expected = [
        "get_start_duel",
        "get_top10_info",
        "get_hryak_info",
        "get_more_info",
        "name_hryak_info",
        "rename_hryak_info",
        "day_pig_info",
        "flag_info",
        "flag_empty_info",
        "flag_change_info",
        "lang_info",
        "lang_empty_info",
        "lang_change_info",
        "cpu_oc_info",
        "ram_oc_info",
        "gpu_oc_info",
        "hru_voice",
        "pig_gif",
        "error_info",
        "error_parse",
        "no_results",
    ];

    let actual: Vec<String> = all_variants()
        .iter()
        .map(|v| {
            v.to_string_with_args()
                .split(InlineResults::DELIMITER)
                .next()
                .unwrap()
                .to_owned()
        })
        .collect();

    assert_eq!(actual, expected);
}


#[test]
fn top10_private_variants_summarize_to_their_public_form() {
    assert_eq!(
        Top10Variant::PGlobal.summarize().into_str(),
        Top10Variant::Global.into_str()
    );
    assert_eq!(
        Top10Variant::PWin.summarize().into_str(),
        Top10Variant::Win.into_str()
    );
}

#[test]
fn top10_public_variants_summarize_to_themselves() {
    for name in ["global", "chat", "win"] {
        let variant = Top10Variant::from_str(name).unwrap();
        assert_eq!(variant.summarize().into_str(), name);
    }
}

#[test]
fn top10_variants_round_trip_through_their_wire_name() {
    let names = ["global", "chat", "win", "p_global", "p_win"];

    for name in names {
        let parsed = Top10Variant::from_str(name)
            .unwrap_or_else(|_| panic!("{name} did not parse"));
        assert_eq!(parsed.into_str(), name);
    }
}
