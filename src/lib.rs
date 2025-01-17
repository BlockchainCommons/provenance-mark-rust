#![doc(html_root_url = "https://docs.rs/provenance-mark/0.2.0")]
#![warn(rust_2018_idioms)]

//! # Introduction
//!
//! [Provenance Marks](https://provemark.com) provide a
//! cryptographically-secured system for establishing and verifying the
//! authenticity of works in an age of rampant AI-powered manipulation and
//! plagiarism. By combining cryptography, pseudorandom number generation, and
//! linguistic representation, this system generates unique, sequential marks
//! that commit to the content of preceding and subsequent works. These marks
//! ensure public and easy verification of provenance, offering robust security
//! and intuitive usability. Provenance Marks are particularly valuable for
//! securing artistic, intellectual, and commercial works against fraud and deep
//! fakes, protecting creators’ reputations and the integrity of their
//! creations.
//!
//! # Getting Started
//!
//! ```toml
//! [dependencies]
//! provenance-mark = "0.2.0"
//! ```
//!
//! # Examples
//!
//! See the unit tests in the source code for examples of how to use this library.

mod resolution;
pub use resolution::*;
mod mark;
pub use mark::*;
mod mark_info;
pub use mark_info::*;
mod generator;
pub use generator::*;
mod seed;
pub use seed::*;
mod rng_state;
pub use rng_state::*;
mod date;
mod crypto_utils;
mod xoshiro256starstar;
pub mod util;
mod envelope;

#[cfg(test)]
mod tests {
    use bc_ur::prelude::*;
    use chrono::TimeZone;
    use dcbor::Date;

    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn run_test(
        resolution: ProvenanceMarkResolution,
        include_info: bool,
        expected_display: &[&str],
        expected_debug: &[&str],
        expected_bytewords: &[&str],
        expected_id_words: &[&str],
        expected_bytemoji_ids: &[&str],
        expected_urs: &[&str],
        expected_urls: &[&str],
    ) {
        crate::register_tags();

        let provenance_gen = ProvenanceMarkGenerator::new_with_passphrase(resolution, "Wolf");
        let count = 10;
        // let base_date = Date::from_string("2023-06-20T12:00:00Z").unwrap();
        let calendar = chrono::Utc;
        let dates: Vec<Date> = (0..count)
            .map(|i| {
                Date::from_datetime(
                    calendar
                        .with_ymd_and_hms(2023, 6, 20, 12, 0, 0)
                        .single()
                        .unwrap()
                        .checked_add_signed(chrono::Duration::days(i))
                        .unwrap()
                )
            })
            .collect();

        let mut encoded_generator = serde_json::to_string(&provenance_gen).unwrap();

        let marks = dates
            .iter()
            .map(|date| {
                let mut gen: ProvenanceMarkGenerator = serde_json
                    ::from_str(&encoded_generator)
                    .unwrap();

                let title = if include_info { Some("Lorem ipsum sit dolor amet.") } else { None };
                let result = gen.next(date.clone(), title);

                encoded_generator = serde_json::to_string(&gen).unwrap();

                result
            })
            .collect::<Vec<_>>();

        assert!(ProvenanceMark::is_sequence_valid(&marks));

        assert!(!marks[1].precedes(&marks[0]));

        if expected_display.is_empty() {
            marks.iter().for_each(|mark| println!(r#""{}","#, mark));
        } else {
            assert_eq!(
                marks
                    .iter()
                    .map(|mark| format!("{}", mark))
                    .collect::<Vec<_>>(),
                    expected_display
            );
        }

        if expected_debug.is_empty() {
            marks.iter().for_each(|mark| println!("{:?}", mark));
        } else {
            assert_eq!(
                marks
                    .iter()
                    .map(|mark| format!("{:?}", mark))
                    .collect::<Vec<_>>(),
                    expected_debug
            );
        }

        let bytewords = marks
            .iter()
            .map(|mark| mark.to_bytewords())
            .collect::<Vec<_>>();
        if expected_bytewords.is_empty() {
            bytewords.iter().for_each(|byteword| println!("{:?}", byteword));
        } else {
            assert_eq!(bytewords, expected_bytewords);
        }
        let bytewords_marks = bytewords
            .iter()
            .map(|byteword| ProvenanceMark::from_bytewords(resolution, byteword).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(marks, bytewords_marks);

        let id_words = marks
            .iter()
            .map(|mark| mark.bytewords_identifier(false))
            .collect::<Vec<_>>();
        if expected_id_words.is_empty() {
            id_words.iter().for_each(|id_word| println!("{:?}", id_word));
        } else {
            assert_eq!(id_words, expected_id_words);
        }

        let bytemoji_ids = marks
            .iter()
            .map(|mark| mark.bytemoji_identifier(false))
            .collect::<Vec<_>>();
        if expected_bytemoji_ids.is_empty() {
            bytemoji_ids.iter().for_each(|bytemoji_id| println!("{:?}", bytemoji_id));
        } else {
            assert_eq!(bytemoji_ids, expected_bytemoji_ids);
        }

        let urs = marks
            .iter()
            .map(|mark| mark.ur_string())
            .collect::<Vec<_>>();
        if expected_urs.is_empty() {
            urs.iter().for_each(|ur| println!("{:?}", ur));
        } else {
            assert_eq!(urs, expected_urs);
        }
        let ur_marks = urs
            .iter()
            .map(|ur| ProvenanceMark::from_ur_string(ur).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(marks, ur_marks);

        let base_url = "https://example.com/validate";
        let urls = marks
            .iter()
            .map(|mark| mark.to_url(base_url))
            .collect::<Vec<_>>();
        if expected_urls.is_empty() {
            urls.iter().for_each(|url| println!("{:?}", url));
        } else {
            assert_eq!(
                urls
                    .iter()
                    .map(|url| url.to_string())
                    .collect::<Vec<_>>(),
                expected_urls
            );
        }
        let url_marks = urls
            .iter()
            .map(|url| ProvenanceMark::from_url(url).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(marks, url_marks);

        for mark in marks.clone() {
            let data = serde_json::to_string(&mark).unwrap();
            let mark2: ProvenanceMark = serde_json::from_str(&data).unwrap();
            assert_eq!(mark, mark2);
        }
    }

    #[test]
    fn test_low() {
        let expected_display = [
            "ProvenanceMark(80485888)",
            "ProvenanceMark(c6faf9f5)",
            "ProvenanceMark(e6cfe72b)",
            "ProvenanceMark(1045e828)",
            "ProvenanceMark(ed855835)",
            "ProvenanceMark(7537da63)",
            "ProvenanceMark(07eccdc1)",
            "ProvenanceMark(7bafde93)",
            "ProvenanceMark(1357eeb2)",
            "ProvenanceMark(0356f83d)",
        ];

        let expected_debug = [
            r#"ProvenanceMark(key: 090bf2f8, hash: 80485888, chainID: 090bf2f8, seq: 0, date: 2023-06-20)"#,
            r#"ProvenanceMark(key: 0ebd3e48, hash: c6faf9f5, chainID: 090bf2f8, seq: 1, date: 2023-06-21)"#,
            r#"ProvenanceMark(key: 774c85bf, hash: e6cfe72b, chainID: 090bf2f8, seq: 2, date: 2023-06-22)"#,
            r#"ProvenanceMark(key: 34e6da59, hash: 1045e828, chainID: 090bf2f8, seq: 3, date: 2023-06-23)"#,
            r#"ProvenanceMark(key: 12167887, hash: ed855835, chainID: 090bf2f8, seq: 4, date: 2023-06-24)"#,
            r#"ProvenanceMark(key: e4fd5d0f, hash: 7537da63, chainID: 090bf2f8, seq: 5, date: 2023-06-25)"#,
            r#"ProvenanceMark(key: efa52ec0, hash: 07eccdc1, chainID: 090bf2f8, seq: 6, date: 2023-06-26)"#,
            r#"ProvenanceMark(key: aac81162, hash: 7bafde93, chainID: 090bf2f8, seq: 7, date: 2023-06-27)"#,
            r#"ProvenanceMark(key: ff383cfa, hash: 1357eeb2, chainID: 090bf2f8, seq: 8, date: 2023-06-28)"#,
            r#"ProvenanceMark(key: bc118ffc, hash: 0356f83d, chainID: 090bf2f8, seq: 9, date: 2023-06-29)"#,
        ];

        let expected_bytewords = [
            "axis bald whiz yoga rich join body jazz cusp kiwi draw echo urge cola exam arch race zest paid foxy",
            "beta ruby film fund zoom able logo fern puma fizz waxy beta iris lion hawk veto swan idle kite keys",
            "kept gems limp runs high need quiz cola tiny zoom buzz acid fact omit visa what good into jury cook",
            "edge visa twin hawk apex brew down echo note brag fact help toil body tiny quad paid need aunt lion",
            "brag calm keys list song owls lamb slot vows road epic news iced gift nail code note dark liar epic",
            "vibe zinc hill bias omit good onyx very ramp pool toil iris jolt warm half inky knob logo memo gear",
            "webs open drum rust visa cola diet vial kiln bald urge door surf gush help yoga road undo open next",
            "peck soap body iced brag echo rust chef foxy rust gush gear kept nail scar jump deli heat solo part",
            "zoom exit fern zaps ramp many join love plus trip bald kiln omit paid jowl blue crux oboe owls slot",
            "roof body many zest brew open junk legs fund real saga math jump tiny lion mint heat omit dice trip",
        ];

        let expected_id_words = [
            "LAVA FUND HARD LOGO",
            "SKEW ZAPS YURT YANK",
            "VISA TASK VOID DOWN",
            "BLUE FREE VOWS DICE",
            "WAVE LIMP HARD EPIC",
            "KEEP EXAM TWIN IDEA",
            "AUNT WASP SWAN SAFE",
            "KING POSE URGE MENU",
            "BREW HANG WAXY PURR",
            "APEX HALF YOGA FIGS",
        ];

        let expected_bytemoji_ids = [
            "💛 🥝 🍔 💯",
            "💥 🦀 🦞 🪽",
            "🐵 🧶 🐔 😿",
            "🥵 🍒 🐥 😻",
            "🐝 🏁 🍔 👄",
            "🌈 👂 🐭 🍦",
            "😍 🦄 🧢 🏈",
            "🌎 📡 🐻 🛵",
            "🤪 🌭 🐛 🧲",
            "😉 🍗 🪼 🍎",
        ];

        let expected_urs = [
            "ur:provenance/lfaegdasbdwzyarhjnbyjzcpkidweouecaemahdizcayhf",
            "ur:provenance/lfaegdbaryfmfdzmaelofnpafzwybaislnhkvoheihuoim",
            "ur:provenance/lfaegdktgslprshhndqzcatyzmbzadftotvawtsaiytybn",
            "ur:provenance/lfaegdeevatnhkaxbwdneonebgfthptlbytyqdftnyosmw",
            "ur:provenance/lfaegdbgcmksltsgoslbstvsrdecnsidgtnlcebtdadkdi",
            "ur:provenance/lfaegdvezchlbsotgdoxvyrppltlisjtwmhfiywpldeyhk",
            "ur:provenance/lfaegdwsondmrtvacadtvlknbduedrsfghhpyadeutahmy",
            "ur:provenance/lfaegdpkspbyidbgeortcffyrtghgrktnlsrjprehpinrk",
            "ur:provenance/lfaegdzmetfnzsrpmyjnlepstpbdknotpdjlbeprotattl",
            "ur:provenance/lfaegdrfbymyztbwonjklsfdrlsamhjptylnmtspoelosg",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfaegdasbdwzyarhjnbyjzcpkidweouecaemahvyqzvtas",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdbaryfmfdzmaelofnpafzwybaislnhkvonldweeec",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdktgslprshhndqzcatyzmbzadftotvawtaadlfngu",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdeevatnhkaxbwdneonebgfthptlbytyqdzttegwsb",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdbgcmksltsgoslbstvsrdecnsidgtnlcesbjzsfks",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdvezchlbsotgdoxvyrppltlisjtwmhfiydrrttnam",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdwsondmrtvacadtvlknbduedrsfghhpyawymwweti",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdpkspbyidbgeortcffyrtghgrktnlsrjpjkbglyve",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdzmetfnzsrpmyjnlepstpbdknotpdjlbejywdwsle",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdrfbymyztbwonjklsfdrlsamhjptylnmtbawmhnmd",
        ];

        run_test(
            ProvenanceMarkResolution::Low,
            false,
            &expected_display,
            &expected_debug,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
        );
    }

    #[test]
    fn test_low_with_info() {
        let expected_display = [
            "ProvenanceMark(9e565482)",
            "ProvenanceMark(34a3c86b)",
            "ProvenanceMark(fc2f3d72)",
            "ProvenanceMark(0da17eef)",
            "ProvenanceMark(ff450e14)",
            "ProvenanceMark(b1dace7e)",
            "ProvenanceMark(c9cbece9)",
            "ProvenanceMark(4d0a0446)",
            "ProvenanceMark(b1b444f9)",
            "ProvenanceMark(43c58c56)",
        ];

        let expected_debug = [
            r#"ProvenanceMark(key: 090bf2f8, hash: 9e565482, chainID: 090bf2f8, seq: 0, date: 2023-06-20, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 0ebd3e48, hash: 34a3c86b, chainID: 090bf2f8, seq: 1, date: 2023-06-21, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 774c85bf, hash: fc2f3d72, chainID: 090bf2f8, seq: 2, date: 2023-06-22, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 34e6da59, hash: 0da17eef, chainID: 090bf2f8, seq: 3, date: 2023-06-23, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 12167887, hash: ff450e14, chainID: 090bf2f8, seq: 4, date: 2023-06-24, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: e4fd5d0f, hash: b1dace7e, chainID: 090bf2f8, seq: 5, date: 2023-06-25, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: efa52ec0, hash: c9cbece9, chainID: 090bf2f8, seq: 6, date: 2023-06-26, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: aac81162, hash: 4d0a0446, chainID: 090bf2f8, seq: 7, date: 2023-06-27, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: ff383cfa, hash: b1b444f9, chainID: 090bf2f8, seq: 8, date: 2023-06-28, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: bc118ffc, hash: 43c58c56, chainID: 090bf2f8, seq: 9, date: 2023-06-29, info: "Lorem ipsum sit dolor amet.")"#,
        ];

        let expected_bytewords = [
            "axis bald whiz yoga rich join body jazz fern idea crux eyes urge cola exam arch girl navy jugs flew unit keys flap very cyan cola flew rock zero jazz yoga owls fair glow film quad runs scar barn glow belt onyx foxy cost apex city into very judo",
            "beta ruby film fund zoom able logo fern flux chef user math iris lion hawk veto taxi luck monk flew jury toys horn dice easy open heat item arch iris omit edge numb work iris body frog brew quad each oboe dull leaf belt ruin help figs also fish",
            "kept gems limp runs high need quiz cola taco cost task hard fact omit visa what list when rock rock wand toys kiln guru flux hard roof frog atom claw fair cola cola play loud purr knob wall dull math flux mild buzz cash jowl redo runs join each",
            "edge visa twin hawk apex brew down echo leaf yawn plus news toil body tiny quad jowl obey body judo inch ruby trip girl deli lamb race yoga duty help exam wasp jury exam owls zone news lung puff diet rock limp yoga fizz view kept jowl tomb news",
            "brag calm keys list song owls lamb slot zaps kiln idea ruby iced gift nail code slot roof gear saga jugs user drum vibe item acid data rust hope calm part rich soap gush hard cusp zoom back slot mint down yell time whiz lazy edge void lava able",
            "vibe zinc hill bias omit good onyx very jump flux safe keep jolt warm half inky good iron jowl bald toil webs yoga pool arch hawk gear gems data idle rock cats dark purr kick back city guru each wasp owls when mild kiln help pool drop zest cost",
            "webs open drum rust visa cola diet vial quiz draw zoom also surf gush help yoga judo love stub limp oboe main miss menu half menu fact yawn flux omit taxi iris dice drop horn back buzz game city yank fizz peck poem tuna pool noon peck kite keep",
            "peck soap body iced brag echo rust chef jump inch main noon kept nail scar jump rich yurt limp into body cost away dark wand note very many vibe road tent yank work need heat gray scar scar belt yell also cats webs door kept echo race zero safe",
            "zoom exit fern zaps ramp many join love beta fair obey each omit paid jowl blue edge dark back dark news back urge ramp girl down onyx main puff runs news dull fair miss belt many buzz limp road brew whiz idle runs each lion zest memo void pool",
            "roof body many zest brew open junk legs away dark ramp zero jump tiny lion mint fund hawk luau leaf lava pool fuel open poem ramp undo idea glow ruby zoom onyx kiln vibe brew trip roof obey math duty arch barn gems flap note guru able need half",
        ];

        let expected_id_words = [
            "NOON HALF GUSH LEAF",
            "EDGE OMIT SOAP JADE",
            "ZEST DULL FIGS JUMP",
            "BELT OBEY KNOB WEBS",
            "ZOOM FREE BETA BULB",
            "PUMA TWIN TACO KNOB",
            "SOLO STUB WASP WALL",
            "GIFT BACK AQUA FROG",
            "PUMA QUIZ FOXY YURT",
            "FLUX SILK LUCK HALF",
        ];

        let expected_bytemoji_ids = [
            "🔔 🍗 🧀 💘",
            "💪 💌 👚 🌹",
            "🦭 🤝 🍎 💧",
            "😶 🪑 🪐 🦋",
            "🐳 🍒 🤨 😵",
            "💰 🐭 👓 🪐",
            "👖 👗 🦄 🦆",
            "🌽 🫠 🙄 🍑",
            "💰 🎁 🫐 🦞",
            "🍓 🔥 🟩 🍗",
        ];

        let expected_urs = [
            "ur:provenance/lfaehddpasbdwzyarhjnbyjzfniacxesuecaemahglnyjsfwutksfpvycncafwrkzojzyaosfrgwfmqdrssrbngwbtoxfyctaxvdayvdjt",
            "ur:provenance/lfaehddpbaryfmfdzmaelofnfxcfurmhislnhkvotilkmkfwjytshndeeyonhtimahisoteenbwkisbyfgbwqdehoedllfbtrnolgmaacl",
            "ur:provenance/lfaehddpktgslprshhndqzcatocttkhdftotvawtltwnrkrkwdtskngufxhdrffgamcwfrcacapyldprkbwldlmhfxmdbzchjlfetijedl",
            "ur:provenance/lfaehddpeevatnhkaxbwdneolfynpsnstlbytyqdjloybyjoihrytpgldilbreyadyhpemwpjyemoszenslgpfdtrklpyafzvwleaetilf",
            "ur:provenance/lfaehddpbgcmksltsgoslbstzskniaryidgtnlcestrfgrsajsurdmveimaddarthecmptrhspghhdcpzmbkstmtdnyltewzlysololnck",
            "ur:provenance/lfaehddpvezchlbsotgdoxvyjpfxsekpjtwmhfiygdinjlbdtlwsyaplahhkgrgsdaierkcsdkprkkbkcyguehwposwnmdknhpgufwzsad",
            "ur:provenance/lfaehddpwsondmrtvacadtvlqzdwzmaosfghhpyajolesblpoemnmsmuhfmuftynfxottiisdedphnbkbzgecyykfzpkpmtapliaskknje",
            "ur:provenance/lfaehddppkspbyidbgeortcfjpihmnnnktnlsrjprhytlpiobyctaydkwdnevymyverdttykwkndhtgysrsrbtylaocswsdrkttotnzcur",
            "ur:provenance/lfaehddpzmetfnzsrpmyjnlebafroyehotpdjlbeeedkbkdknsbkuerpgldnoxmnpfrsnsdlfrmsbtmybzlprdbwwziersehlnadzcvypf",
            "ur:provenance/lfaehddprfbymyztbwonjklsaydkrpzojptylnmtfdhklulflaplflonpmrpuoiagwryzmoxknvebwtprfoymhdyahbngsfpnepljlntfd",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpasbdwzyarhjnbyjzfniacxesuecaemahglnyjsfwutksfpvycncafwrkzojzyaosfrgwfmqdrssrbngwbtoxfyctaxaeemwkla",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpbaryfmfdzmaelofnfxcfurmhislnhkvotilkmkfwjytshndeeyonhtimahisoteenbwkisbyfgbwqdehoedllfbtrnfpjnchtk",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpktgslprshhndqzcatocttkhdftotvawtltwnrkrkwdtskngufxhdrffgamcwfrcacapyldprkbwldlmhfxmdbzchjloewsksse",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpeevatnhkaxbwdneolfynpsnstlbytyqdjloybyjoihrytpgldilbreyadyhpemwpjyemoszenslgpfdtrklpyafzvwjnfhsrjz",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpbgcmksltsgoslbstzskniaryidgtnlcestrfgrsajsurdmveimaddarthecmptrhspghhdcpzmbkstmtdnyltewzlydmrlmdwt",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpvezchlbsotgdoxvyjpfxsekpjtwmhfiygdinjlbdtlwsyaplahhkgrgsdaierkcsdkprkkbkcyguehwposwnmdknhpqzkiwlws",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpwsondmrtvacadtvlqzdwzmaosfghhpyajolesblpoemnmsmuhfmuftynfxottiisdedphnbkbzgecyykfzpkpmtapllrzsinlp",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddppkspbyidbgeortcfjpihmnnnktnlsrjprhytlpiobyctaydkwdnevymyverdttykwkndhtgysrsrbtylaocswsdrktdtvwwyeh",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpzmetfnzsrpmyjnlebafroyehotpdjlbeeedkbkdknsbkuerpgldnoxmnpfrsnsdlfrmsbtmybzlprdbwwziersehlnvasawzhy",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddprfbymyztbwonjklsaydkrpzojptylnmtfdhklulflaplflonpmrpuoiagwryzmoxknvebwtprfoymhdyahbngsfpnegagdmnol",
        ];

        run_test(
            ProvenanceMarkResolution::Low,
            true,
            &expected_display,
            &expected_debug,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
        );
    }

    #[test]
    fn test_medium() {
        let expected_display = [
            "ProvenanceMark(10a0e9a7)",
            "ProvenanceMark(ab3781a2)",
            "ProvenanceMark(c041ec3a)",
            "ProvenanceMark(191a067c)",
            "ProvenanceMark(5b27040e)",
            "ProvenanceMark(0395c4f0)",
            "ProvenanceMark(de4598b6)",
            "ProvenanceMark(e8689256)",
            "ProvenanceMark(59cb1f69)",
            "ProvenanceMark(36b71f95)",
        ];

        let expected_debug = [
            r#"ProvenanceMark(key: 090bf2f8b55be45b, hash: 10a0e9a772cebde7, chainID: 090bf2f8b55be45b, seq: 0, date: 2023-06-20T12:00:00Z)"#,
            r#"ProvenanceMark(key: 0ebd3e48774c85bf, hash: ab3781a29f16485a, chainID: 090bf2f8b55be45b, seq: 1, date: 2023-06-21T12:00:00Z)"#,
            r#"ProvenanceMark(key: 34e6da5912167887, hash: c041ec3adb7e4320, chainID: 090bf2f8b55be45b, seq: 2, date: 2023-06-22T12:00:00Z)"#,
            r#"ProvenanceMark(key: e4fd5d0fefa52ec0, hash: 191a067ca3c5dfe4, chainID: 090bf2f8b55be45b, seq: 3, date: 2023-06-23T12:00:00Z)"#,
            r#"ProvenanceMark(key: aac81162ff383cfa, hash: 5b27040ea87ed756, chainID: 090bf2f8b55be45b, seq: 4, date: 2023-06-24T12:00:00Z)"#,
            r#"ProvenanceMark(key: bc118ffc1221462d, hash: 0395c4f06079c050, chainID: 090bf2f8b55be45b, seq: 5, date: 2023-06-25T12:00:00Z)"#,
            r#"ProvenanceMark(key: 1052822be985b2c4, hash: de4598b6b72d6444, chainID: 090bf2f8b55be45b, seq: 6, date: 2023-06-26T12:00:00Z)"#,
            r#"ProvenanceMark(key: c6c75df5a9b9baab, hash: e868925698fcadb6, chainID: 090bf2f8b55be45b, seq: 7, date: 2023-06-27T12:00:00Z)"#,
            r#"ProvenanceMark(key: 942b5fa11fb5b285, hash: 59cb1f693bbf6fc3, chainID: 090bf2f8b55be45b, seq: 8, date: 2023-06-28T12:00:00Z)"#,
            r#"ProvenanceMark(key: 9b25daa7646063a7, hash: 36b71f9581fcf5ca, chainID: 090bf2f8b55be45b, seq: 9, date: 2023-06-29T12:00:00Z)"#,
        ];

        let expected_bytewords = [
            "axis bald whiz yoga race help vibe help scar many list undo buzz puma urge hawk play runs whiz keno exit slot vows quiz slot back back vial join play days open lion lung surf down",
            "beta ruby film fund kept gems limp runs each cook need redo blue girl away lazy kick cola solo judo redo onyx able jugs wand idle redo idea taxi axis veto runs easy crux item gems",
            "edge visa twin hawk brag calm keys list drum navy quad epic nail body kept monk gear flap idea monk limp idea jade very brag pose door zone wand next kite wave news hang glow draw",
            "vibe zinc hill bias webs open drum rust logo oboe purr fund wave each sets acid aunt work good iron lamb beta able junk mild lamb vast twin heat good mild diet what drop yell days",
            "peck soap body iced zoom exit fern zaps fair waxy iris many fizz duty good cusp belt jump roof part task iris puff next luau fund vast cusp aunt grim limp saga poem rust jolt yoga",
            "roof body many zest brag curl frog drop hawk city wolf zinc yank yawn yoga easy stub view zoom love arch yoga ruin need keno twin each surf zinc limp veto jazz legs king kiwi list",
            "blue grim leaf down wall limp purr sets fern cats back half kept liar dice hope part iron city meow navy belt flux draw exam keys ruby cook huts jury toys tent tuna noon maze film",
            "skew slot hill yank part rich road play film oval kept onyx figs claw fair noon toys taco game rich fact jowl grim inky note aunt tiny user tuna math crux kick loud tied gems door",
            "meow down hope obey cost race purr limp apex leaf veto many inch figs figs puma help news wasp miss visa race bulb waxy fair main brew what edge luck race iron vial lung tiny user",
            "need data twin owls idle horn idea owls barn saga warm bulb quad good drop scar task axis iron ruby hang part yank epic yurt pool tent acid luau poem surf jazz fact eyes fern also",
        ];

        let expected_id_words = [
            "BLUE NUMB WALL OWLS",
            "PLAY EXAM LAZY OBOE",
            "RUST FLAP WASP FACT",
            "CHEF CITY ATOM KITE",
            "HELP DELI AQUA BETA",
            "APEX MILD SETS WHAT",
            "URGE FREE MONK RAMP",
            "VOWS IRIS MEMO HALF",
            "HAWK STUB COST IRON",
            "EVEN REAL COST MILD",
        ];

        let expected_bytemoji_ids = [
            "🥵 🚪 🦆 📚",
            "💎 👂 💔 🎈",
            "🏀 🍉 🦄 👀",
            "🤡 🥳 😎 💫",
            "🌮 😹 🙄 🤨",
            "😉 🚀 ✨ 🐌",
            "🐻 🍒 🚦 🎉",
            "🐥 💐 🚜 🍗",
            "🍟 👗 🤯 🍁",
            "🦷 🪭 🤯 🚀",
        ];

        let expected_urs = [
            "ur:provenance/lfadhdcxasbdwzyarehpvehpsrmyltuobzpauehkpyrswzkoetstvsqzstbkbkvljnpydsonlocyinws",
            "ur:provenance/lfadhdcxbaryfmfdktgslprsehckndrobeglaylykkcasojorooxaejswdieroiatiasvorsfnrltklo",
            "ur:provenance/lfadhdcxeevatnhkbgcmksltdmnyqdecnlbyktmkgrfpiamklpiajevybgpedrzewdntkewemortwdvs",
            "ur:provenance/lfadhdcxvezchlbswsondmrtlooeprfdweehssadatwkgdinlbbaaejkmdlbvttnhtgdmddtzerdgmvo",
            "ur:provenance/lfadhdcxpkspbyidzmetfnzsfrwyismyfzdygdcpbtjprfpttkispfntlufdvtcpatgmlpsaothgsbfn",
            "ur:provenance/lfadhdcxrfbymyztbgclfgdphkcywfzcykynyaeysbvwzmleahyarnndkotnehsfzclpvojzlgwptpfx",
            "ur:provenance/lfadhdcxbegmlfdnwllpprssfncsbkhfktlrdeheptincymwnybtfxdwemksryckhsjytstttsaseezs",
            "ur:provenance/lfadhdcxswsthlykptrhrdpyfmolktoxfscwfrnntstogerhftjlgmiyneattyurtamhcxkkltfewlwy",
            "ur:provenance/lfadhdcxmwdnheoyctreprlpaxlfvomyihfsfspahpnswpmsvarebbwyfrmnbwwteelkreinwecyjscw",
            "ur:provenance/lfadhdcxnddatnosiehniaosbnsawmbbqdgddpsrtkasinryhgptykecytplttadlupmsfjzeeplnlsw",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxasbdwzyarehpvehpsrmyltuobzpauehkpyrswzkoetstvsqzstbkbkvljnpydsonmwoyiyet",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxbaryfmfdktgslprsehckndrobeglaylykkcasojorooxaejswdieroiatiasvorscxbnrthe",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxeevatnhkbgcmksltdmnyqdecnlbyktmkgrfpiamklpiajevybgpedrzewdntkewemnkgvwfh",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxvezchlbswsondmrtlooeprfdweehssadatwkgdinlbbaaejkmdlbvttnhtgdmddtvoadhlec",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxpkspbyidzmetfnzsfrwyismyfzdygdcpbtjprfpttkispfntlufdvtcpatgmlpsarswpsswm",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxrfbymyztbgclfgdphkcywfzcykynyaeysbvwzmleahyarnndkotnehsfzclpvojzmehgtsmw",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxbegmlfdnwllpprssfncsbkhfktlrdeheptincymwnybtfxdwemksryckhsjytsttsbprfrdp",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxswsthlykptrhrdpyfmolktoxfscwfrnntstogerhftjlgmiyneattyurtamhcxkkndzevaes",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxmwdnheoyctreprlpaxlfvomyihfsfspahpnswpmsvarebbwyfrmnbwwteelkreinwnoykbsf",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxnddatnosiehniaosbnsawmbbqdgddpsrtkasinryhgptykecytplttadlupmsfjzdebzmtby",
        ];

        run_test(
            ProvenanceMarkResolution::Medium,
            false,
            &expected_display,
            &expected_debug,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
        );
    }

    #[test]
    fn test_medium_with_info() {
        let expected_display = [
            "ProvenanceMark(70b1ec44)",
            "ProvenanceMark(0b33dfc7)",
            "ProvenanceMark(df6452b2)",
            "ProvenanceMark(2c4b2ea6)",
            "ProvenanceMark(38e71cf8)",
            "ProvenanceMark(682b68f0)",
            "ProvenanceMark(f50ea891)",
            "ProvenanceMark(0eed8c3f)",
            "ProvenanceMark(2cea6751)",
            "ProvenanceMark(e54dd475)",
        ];

        let expected_debug = [
            r#"ProvenanceMark(key: 090bf2f8b55be45b, hash: 70b1ec44fe52d618, chainID: 090bf2f8b55be45b, seq: 0, date: 2023-06-20T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 0ebd3e48774c85bf, hash: 0b33dfc7d2e1a971, chainID: 090bf2f8b55be45b, seq: 1, date: 2023-06-21T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 34e6da5912167887, hash: df6452b208051069, chainID: 090bf2f8b55be45b, seq: 2, date: 2023-06-22T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: e4fd5d0fefa52ec0, hash: 2c4b2ea6ce4db2d2, chainID: 090bf2f8b55be45b, seq: 3, date: 2023-06-23T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: aac81162ff383cfa, hash: 38e71cf83104e361, chainID: 090bf2f8b55be45b, seq: 4, date: 2023-06-24T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: bc118ffc1221462d, hash: 682b68f04b03ce61, chainID: 090bf2f8b55be45b, seq: 5, date: 2023-06-25T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 1052822be985b2c4, hash: f50ea891889e00d5, chainID: 090bf2f8b55be45b, seq: 6, date: 2023-06-26T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: c6c75df5a9b9baab, hash: 0eed8c3f2e9d761f, chainID: 090bf2f8b55be45b, seq: 7, date: 2023-06-27T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 942b5fa11fb5b285, hash: 2cea6751fe4745d9, chainID: 090bf2f8b55be45b, seq: 8, date: 2023-06-28T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 9b25daa7646063a7, hash: e54dd4753bb65e5b, chainID: 090bf2f8b55be45b, seq: 9, date: 2023-06-29T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
        ];

        let expected_bytewords = [
            "axis bald whiz yoga race help vibe help scar many list undo buzz puma urge hawk stub pool yell mild quiz help legs gear slot back back vial join play days open void mint visa help grim peck waxy jowl tuna play onyx yank many fuel brag cash brew girl tiny arch webs very vial lamb safe owls iron onyx fair obey apex jugs acid",
            "beta ruby film fund kept gems limp runs each cook need redo blue girl away lazy tuna chef miss buzz yank guru very heat wand idle redo idea taxi axis veto runs purr calm safe wand jump each wasp keep jazz hard aqua belt down buzz city navy into rock holy memo menu gems quiz each jury yell very belt diet love yurt unit limp",
            "edge visa twin hawk brag calm keys list drum navy quad epic nail body kept monk gush idle unit blue half cats exit paid brag pose door zone wand next kite wave user vial gift hang huts gray iced guru unit hill knob girl body leaf able fern bulb noon fuel keno data fish guru sets fern safe gush part paid fizz ramp down days",
            "vibe zinc hill bias webs open drum rust logo oboe purr fund wave each sets acid easy open keys quad brag lion join free mild lamb vast twin heat good mild diet yoga paid veto open girl omit next judo safe monk back warm keno hang jolt draw toys list vast menu jowl vibe calm kiln tuna iced aunt into high figs even high cook",
            "peck soap body iced zoom exit fern zaps fair waxy iris many fizz duty good cusp jolt purr onyx hope half brag liar peck luau fund vast cusp aunt grim limp saga jazz buzz jade leaf stub twin knob vast fizz iron yurt visa roof vibe ruby kick fair flap undo hill girl lung fair peck eyes dull zest crux dull hill vial cats fizz",
            "roof body many zest brag curl frog drop hawk city wolf zinc yank yawn yoga easy numb help guru love drum leaf puff peck keno twin each surf zinc limp veto jazz peck gift crux oval liar hard acid half cost tent redo stub fund gift tuna free vows song menu roof inky toil judo fish gear help vibe beta unit mild open heat fund",
            "blue grim leaf down wall limp purr sets fern cats back half kept liar dice hope leaf cusp door quad open ruin deli ruby exam keys ruby cook huts jury toys tent ugly swan grim kiln mild iris kite lava draw zone pool liar miss half dull scar jump open trip next door film atom cusp hang jugs kick kiln mint hard tied fern able",
            "skew slot hill yank part rich road play film oval kept onyx figs claw fair noon each gear gush taxi luck beta loud task note aunt tiny user tuna math crux kick hill cash luck cash drop yell horn vows also jolt easy zoom glow cola idle jugs lazy tomb crux lava menu horn silk user back apex work cyan half time play roof junk",
            "meow down hope obey cost race purr limp apex leaf veto many inch figs figs puma drum ruby meow pose cyan gift film work fair main brew what edge luck race iron list lion next idea glow atom jugs maze song jump gray yurt chef maze exit fuel hope tiny huts huts yawn fizz yank luck cash tiny jump brag huts calm purr door free",
            "need data twin owls idle horn idea owls barn saga warm bulb quad good drop scar code wolf oboe hill wave vial holy onyx yurt pool tent acid luau poem surf jazz bias omit vibe able purr song ruin omit meow void runs inky girl very yell figs luau user hope cook need news brag need fund gems yawn mild legs claw kept jowl dull",
        ];

        let expected_id_words = [
            "JUDO PUMA WASP FOXY",
            "BALD ECHO USER SLOT",
            "USER IDLE GRIM PURR",
            "DRAW GEAR DRUM OVAL",
            "EXIT VOID CODE YOGA",
            "IRIS DOWN IRIS WHAT",
            "YANK BETA PAID MAZE",
            "BETA WAVE LUCK FISH",
            "DRAW WAND INTO GRAY",
            "VIEW GIFT TINY KEEP",
        ];

        let expected_bytemoji_ids = [
            "💨 💰 🦄 🫐",
            "🥱 👆 🐼 👕",
            "🐼 🎂 🥯 🧲",
            "🫶 🥦 🙌 📖",
            "👃 🐔 😬 🪼",
            "💐 😿 💐 🐌",
            "🪽 🤨 📌 🚒",
            "🤨 🐝 🟩 🍋",
            "🫶 🦉 🌱 🥐",
            "🐸 🌽 🧦 🌈",
        ];

        let expected_urs = [
            "ur:provenance/lfadhdfsasbdwzyarehpvehpsrmyltuobzpauehksbplylmdqzhplsgrstbkbkvljnpydsonvdmtvahpgmpkwyjltapyoxykmyflbgchbwgltyahwsvyvllbseosinoxfrylwfiepl",
            "ur:provenance/lfadhdfsbaryfmfdktgslprsehckndrobeglaylytacfmsbzykguvyhtwdieroiatiasvorsprcmsewdjpehwpkpjzhdaabtdnbzcynyiorkhymomugsqzehjyylvybtdtuoasspdr",
            "ur:provenance/lfadhdfseevatnhkbgcmksltdmnyqdecnlbyktmkghieutbehfcsetpdbgpedrzewdntkeweurvlgthghsgyidguuthlkbglbylfaefnbbnnflkodafhgussfnseghptpdcmfgfmld",
            "ur:provenance/lfadhdfsvezchlbswsondmrtlooeprfdweehssadeyonksqdbglnjnfemdlbvttnhtgdmddtyapdvoonglotntjosemkbkwmkohgjtdwtsltvtmujlvecmkntaidatiohhjeswgapa",
            "ur:provenance/lfadhdfspkspbyidzmetfnzsfrwyismyfzdygdcpjtproxhehfbglrpklufdvtcpatgmlpsajzbzjelfsbtnkbvtfzinytvarfverykkfrfpuohlgllgfrpkesdlztcxdlbdbwbtws",
            "ur:provenance/lfadhdfsrfbymyztbgclfgdphkcywfzcykynyaeynbhpguledmlfpfpkkotnehsfzclpvojzpkgtcxollrhdadhfctttrosbfdgttafevssgmurfiytljofhgrhpvebautsrgogwvd",
            "ur:provenance/lfadhdfsbegmlfdnwllpprssfncsbkhfktlrdehelfcpdrqdonrndiryemksryckhsjytsttuysngmknmdiskeladwzepllrmshfdlsrjpontpntdrfmamcphgjskkknmtbacpdtpe",
            "ur:provenance/lfadhdfsswsthlykptrhrdpyfmolktoxfscwfrnnehgrghtilkbaldtkneattyurtamhcxkkhlchlkchdpylhnvsaojteyzmgwcaiejslytbcxlamuhnskurbkaxwkcnhflphpptuo",
            "ur:provenance/lfadhdfsmwdnheoyctreprlpaxlfvomyihfsfspadmrymwpecngtfmwkfrmnbwwteelkreinltlnntiagwamjsmesgjpgyytcfmeetflhetyhshsynfzyklkchtyjpbghsfzfwfhwd",
            "ur:provenance/lfadhdfsnddatnosiehniaosbnsawmbbqdgddpsrcewfoehlwevlhyoxytplttadlupmsfjzbsotveaeprsgrnotmwvdrsiyglvyylfsluurheckndnsbgndfdgsynmdlsgtltknla",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsasbdwzyarehpvehpsrmyltuobzpauehksbplylmdqzhplsgrstbkbkvljnpydsonvdmtvahpgmpkwyjltapyoxykmyflbgchbwgltyahwsvyvllbseosinoxfrsoknbabw",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsbaryfmfdktgslprsehckndrobeglaylytacfmsbzykguvyhtwdieroiatiasvorsprcmsewdjpehwpkpjzhdaabtdnbzcynyiorkhymomugsqzehjyylvybtdtvolaoems",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfseevatnhkbgcmksltdmnyqdecnlbyktmkghieutbehfcsetpdbgpedrzewdntkeweurvlgthghsgyidguuthlkbglbylfaefnbbnnflkodafhgussfnseghptpddetkghee",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsvezchlbswsondmrtlooeprfdweehssadeyonksqdbglnjnfemdlbvttnhtgdmddtyapdvoonglotntjosemkbkwmkohgjtdwtsltvtmujlvecmkntaidatiohhgogwcnbn",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfspkspbyidzmetfnzsfrwyismyfzdygdcpjtproxhehfbglrpklufdvtcpatgmlpsajzbzjelfsbtnkbvtfzinytvarfverykkfrfpuohlgllgfrpkesdlztcxdlecnyiogm",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsrfbymyztbgclfgdphkcywfzcykynyaeynbhpguledmlfpfpkkotnehsfzclpvojzpkgtcxollrhdadhfctttrosbfdgttafevssgmurfiytljofhgrhpvebautzcuodaht",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsbegmlfdnwllpprssfncsbkhfktlrdehelfcpdrqdonrndiryemksryckhsjytsttuysngmknmdiskeladwzepllrmshfdlsrjpontpntdrfmamcphgjskkknmtdypyfxbg",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsswsthlykptrhrdpyfmolktoxfscwfrnnehgrghtilkbaldtkneattyurtamhcxkkhlchlkchdpylhnvsaojteyzmgwcaiejslytbcxlamuhnskurbkaxwkcnhfrktdsrhs",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsmwdnheoyctreprlpaxlfvomyihfsfspadmrymwpecngtfmwkfrmnbwwteelkreinltlnntiagwamjsmesgjpgyytcfmeetflhetyhshsynfzyklkchtyjpbghskbsbgohg",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsnddatnosiehniaosbnsawmbbqdgddpsrcewfoehlwevlhyoxytplttadlupmsfjzbsotveaeprsgrnotmwvdrsiyglvyylfsluurheckndnsbgndfdgsynmdlsjkbabefs",
        ];

        run_test(
            ProvenanceMarkResolution::Medium,
            true,
            &expected_display,
            &expected_debug,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
        );
    }

    #[test]
    fn test_quartile() {
        let expected_display = [
            "ProvenanceMark(f3796db4)",
            "ProvenanceMark(2b533fe9)",
            "ProvenanceMark(a2f9d48e)",
            "ProvenanceMark(2f57f20c)",
            "ProvenanceMark(253417d9)",
            "ProvenanceMark(3f172f0e)",
            "ProvenanceMark(731c1e59)",
            "ProvenanceMark(2ec10f10)",
            "ProvenanceMark(da8bafca)",
            "ProvenanceMark(fc0f4362)",
        ];

        let expected_debug = [
            r#"ProvenanceMark(key: 090bf2f8b55be45b4661b24b7e9c340c, hash: f3796db49d744e0f6f91ac8914c6f44f, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 0, date: 2023-06-20T12:00:00Z)"#,
            r#"ProvenanceMark(key: 0ebd3e48774c85bf34e6da5912167887, hash: 2b533fe94425e17afcc4290b4b43129e, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 1, date: 2023-06-21T12:00:00Z)"#,
            r#"ProvenanceMark(key: e4fd5d0fefa52ec0aac81162ff383cfa, hash: a2f9d48ee48a291baa64d0f28a96c39b, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 2, date: 2023-06-22T12:00:00Z)"#,
            r#"ProvenanceMark(key: bc118ffc1221462d1052822be985b2c4, hash: 2f57f20c5e08ec73c3031c84b3ae982c, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 3, date: 2023-06-23T12:00:00Z)"#,
            r#"ProvenanceMark(key: c6c75df5a9b9baab942b5fa11fb5b285, hash: 253417d98e64b7e7670051503b6d8bf1, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 4, date: 2023-06-24T12:00:00Z)"#,
            r#"ProvenanceMark(key: 9b25daa7646063a7183df7d73fc84901, hash: 3f172f0e743bcdf356684aa58224a60e, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 5, date: 2023-06-25T12:00:00Z)"#,
            r#"ProvenanceMark(key: bfc1cbdf1a44ff9e3cc2e4343cd83f36, hash: 731c1e59ad813545ae683d52942963d0, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 6, date: 2023-06-26T12:00:00Z)"#,
            r#"ProvenanceMark(key: 389e72a009ffec91036ce49a8a3685ad, hash: 2ec10f1000a7b4111cd60aa9b9309724, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 7, date: 2023-06-27T12:00:00Z)"#,
            r#"ProvenanceMark(key: 7b2b6554818a402aef8b285d969663f1, hash: da8bafca881506b91ae62d76207ed64d, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 8, date: 2023-06-28T12:00:00Z)"#,
            r#"ProvenanceMark(key: e6b18e4fa1625d4e90c3a200e92f8f3f, hash: fc0f43627cbb69e561a16c41474d3543, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 9, date: 2023-06-29T12:00:00Z)"#,
        ];

        let expected_bytewords = [
            "axis bald whiz yoga race help vibe help frog huts purr gear knob news edge barn inky jump mild warm warm pose obey ruby very hill yank song frog into maze work main news body knob echo knob ugly holy main able glow user claw maze menu down menu peck fern fizz item cost rich mild nail omit swan gala toys atom",
            "beta ruby film fund kept gems limp runs edge visa twin hawk brag calm keys list lion iron luck puff huts city loud exam peck song tuna quad vibe void wall road open scar when rock poem cost many fuel unit atom roof curl days lung heat item data epic logo days what huts work main echo wolf vial silk yurt view",
            "vibe zinc hill bias webs open drum rust peck soap body iced zoom exit fern zaps beta frog hill pool away fact frog dice aunt huts time girl paid taxi paid void easy zinc aunt soap vibe plus tent play work into play meow silk cook gray quad open monk purr jowl luck numb diet time surf claw main omit time roof",
            "roof body many zest brag curl frog drop blue grim leaf down wall limp purr sets join item deli figs away zone poem when idle frog jolt play next crux hang fact film owls tiny hard urge junk wall wasp luau judo bias body surf tied what luau aunt rust noon guru iron claw hill also drop view wave foxy deli iron",
            "skew slot hill yank part rich road play meow down hope obey cost race purr limp apex note buzz belt echo skew hawk trip junk days keep memo half swan hang cusp roof play calm fern girl quad wall free zone next webs lion cusp days also onyx king zero wasp free view whiz arch road acid also toil good zone meow",
            "need data twin owls idle horn idea owls cats figs yell toys fish soap gala acid tuna hang hill game zoom king owls king oboe news safe runs monk gift blue idle kept soap gyro blue real mild ruby safe door loud judo fund chef cola gush void film guru undo gift ruin item wall open body barn liar iron kite film",
            "runs safe stub user city foxy zoom noon fern saga vibe edge fern trip fish even pool exit liar purr fish luau figs cook legs time also apex into cash nail item tiny puff duty liar trip draw gear legs grim safe logo join dice menu limp data junk yoga acid saga poem into cyan omit each wolf gray knob keno time",
            "exit noon jump numb axis zoom wasp maze apex jazz vibe navy love even limp poem open beta logo slot paid chef join poem city quiz away whiz drop barn claw jugs bald quad high stub skew high body oval item gift door owls cats kiwi cook idea echo runs navy iron item quad navy exit junk navy purr time guru taxi",
            "king down inch gush lazy love fizz door webs luau dice hill mint mint idea when gems door safe need visa inch guru noon legs omit monk unit back crux whiz days cash bulb cola love huts noon love obey inky logo menu menu tuna holy toys wasp obey iris gray dark flux paid gear kite arch silk waxy wand race quad",
            "visa puma main glow obey iced hill girl math scar oboe able wall dull many fish puma yurt fact jazz pose plus visa keep fish iris wand tiny runs brew brew brag brag pose tuna rich able open poem tent toys blue vows cook bulb oboe good paid solo bald limp luau surf epic calm hope blue obey maze liar yank also",
        ];

        let expected_id_words = [
            "WOLF KICK JOIN QUIZ",
            "DOWN GURU FISH WALL",
            "OBOE YURT TINY MAIN",
            "DULL HANG WHIZ BARN",
            "DATA EDGE CASH TUNA",
            "FISH CASH DULL BETA",
            "JUNK CODE COOK HAWK",
            "DRUM SAFE BIAS BLUE",
            "TWIN LUAU POSE SONG",
            "ZEST BIAS FLUX ICED",
        ];

        let expected_bytemoji_ids = [
            "🐺 🌜 🌼 🎁",
            "😿 🍞 🍋 🦆",
            "🎈 🦞 🧦 🔺",
            "🤝 🌭 🐢 🤩",
            "👽 💪 😇 🐶",
            "🍋 😇 🤝 🤨",
            "💦 😬 🙃 🍟",
            "🙌 🏈 🫥 🥵",
            "🐭 🔷 📡 🩳",
            "🦭 🫥 🍓 🍨",
        ];

        let expected_urs = [
            "ur:provenance/lfaohdftasbdwzyarehpvehpfghsprgrkbnseebniyjpmdwmwmpeoyryvyhlyksgfgiomewkmnnsbykbeokbuyhymnaegwurcwmemudnmupkfnfzimctrhmdnlothfjsksva",
            "ur:provenance/lfaohdftbaryfmfdktgslprseevatnhkbgcmksltlninlkpfhscyldempksgtaqdvevdwlrdonsrwnrkpmctmyflutamrfcldslghtimdaeclodswthswkmneowfkszchfah",
            "ur:provenance/lfaohdftvezchlbswsondmrtpkspbyidzmetfnzsbafghlplayftfgdeathsteglpdtipdvdeyzcatspvepsttpywkiopymwskckgyqdonmkprjllknbdttesfcwbzndkehh",
            "ur:provenance/lfaohdftrfbymyztbgclfgdpbegmlfdnwllpprssjnimdifsayzepmwniefgjtpyntcxhgftfmostyhduejkwlwplujobsbysftdwtluatrtnnguincwhlaodpvwkokelold",
            "ur:provenance/lfaohdftswsthlykptrhrdpymwdnheoyctreprlpaxnebzbteoswhktpjkdskpmohfsnhgcprfpycmfnglqdwlfezentwslncpdsaooxkgzowpfevwwzahrdadaoglisgyjy",
            "ur:provenance/lfaohdftnddatnosiehniaoscsfsyltsfhspgaadtahghlgezmkgoskgoenssersmkgtbeiektspgoberlmdrysedrldjofdcfcaghvdfmguuogtrnimwlonbybnctgyteue",
            "ur:provenance/lfaohdftrssesburcyfyzmnnfnsaveeefntpfhenpletlrprfhlufscklsteaoaxiochnlimtypfdylrtpdwgrlsgmselojndemulpdajkyaadsapmiocnotehwfsgfgtaeo",
            "ur:provenance/lfaohdftetnnjpnbaszmwpmeaxjzvenyleenlppmonbalostpdcfjnpmcyqzaywzdpbncwjsbdqdhhsbswhhbyolimgtdroscskickiaeorsnyinimqdnyetjknydtwmztdy",
            "ur:provenance/lfaohdftkgdnihghlylefzdrwsludehlmtmtiawngsdrsendvaihgunnlsotmkutbkcxwzdschbbcalehsnnleoyiylomumutahytswpoyisgydkfxpdgrkeahskkptdcygu",
            "ur:provenance/lfaohdftvapamngwoyidhlglmhsroeaewldlmyfhpaytftjzpepsvakpfhiswdtyrsbwbwbgbgpetarhaeonpmtttsbevsckbboegdpdsobdlplusfeccmhebeoybkrfhtvo",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftasbdwzyarehpvehpfghsprgrkbnseebniyjpmdwmwmpeoyryvyhlyksgfgiomewkmnnsbykbeokbuyhymnaegwurcwmemudnmupkfnfzimctrhmdnlotchdkcase",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftbaryfmfdktgslprseevatnhkbgcmksltlninlkpfhscyldempksgtaqdvevdwlrdonsrwnrkpmctmyflutamrfcldslghtimdaeclodswthswkmneowfespdeocp",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftvezchlbswsondmrtpkspbyidzmetfnzsbafghlplayftfgdeathsteglpdtipdvdeyzcatspvepsttpywkiopymwskckgyqdonmkprjllknbdttesfcwghtocfkg",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftrfbymyztbgclfgdpbegmlfdnwllpprssjnimdifsayzepmwniefgjtpyntcxhgftfmostyhduejkwlwplujobsbysftdwtluatrtnnguincwhlaodpvwemdtwepl",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftswsthlykptrhrdpymwdnheoyctreprlpaxnebzbteoswhktpjkdskpmohfsnhgcprfpycmfnglqdwlfezentwslncpdsaooxkgzowpfevwwzahrdadaobsfseegu",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftnddatnosiehniaoscsfsyltsfhspgaadtahghlgezmkgoskgoenssersmkgtbeiektspgoberlmdrysedrldjofdcfcaghvdfmguuogtrnimwlonbybnhyaarpyt",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftrssesburcyfyzmnnfnsaveeefntpfhenpletlrprfhlufscklsteaoaxiochnlimtypfdylrtpdwgrlsgmselojndemulpdajkyaadsapmiocnotehwflubwrfbb",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftetnnjpnbaszmwpmeaxjzvenyleenlppmonbalostpdcfjnpmcyqzaywzdpbncwjsbdqdhhsbswhhbyolimgtdroscskickiaeorsnyinimqdnyetjknyisrnnlch",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftkgdnihghlylefzdrwsludehlmtmtiawngsdrsendvaihgunnlsotmkutbkcxwzdschbbcalehsnnleoyiylomumutahytswpoyisgydkfxpdgrkeahskeeltlbjy",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftvapamngwoyidhlglmhsroeaewldlmyfhpaytftjzpepsvakpfhiswdtyrsbwbwbgbgpetarhaeonpmtttsbevsckbboegdpdsobdlplusfeccmhebeoygrwlfhsk",
        ];

        run_test(
            ProvenanceMarkResolution::Quartile,
            false,
            &expected_display,
            &expected_debug,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
        );
    }

    #[test]
    fn test_quartile_with_info() {
        let expected_display = [
            "ProvenanceMark(1aaf875f)",
            "ProvenanceMark(4abee006)",
            "ProvenanceMark(8105a094)",
            "ProvenanceMark(e171f1ee)",
            "ProvenanceMark(d27a7296)",
            "ProvenanceMark(167025b1)",
            "ProvenanceMark(0af3c960)",
            "ProvenanceMark(3c04aec0)",
            "ProvenanceMark(49b22ab6)",
            "ProvenanceMark(ad2d74d2)",
        ];

        let expected_debug = [
            r#"ProvenanceMark(key: 090bf2f8b55be45b4661b24b7e9c340c, hash: 1aaf875f91a6af24bbfe6a726fcda33e, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 0, date: 2023-06-20T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 0ebd3e48774c85bf34e6da5912167887, hash: 4abee0064bb6eaf4683831618699f278, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 1, date: 2023-06-21T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: e4fd5d0fefa52ec0aac81162ff383cfa, hash: 8105a094d6a8662819d5d7407ad733c8, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 2, date: 2023-06-22T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: bc118ffc1221462d1052822be985b2c4, hash: e171f1eebcafe8d68999cdee24dc8cb9, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 3, date: 2023-06-23T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: c6c75df5a9b9baab942b5fa11fb5b285, hash: d27a72961b0c81e31384909a305424d2, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 4, date: 2023-06-24T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 9b25daa7646063a7183df7d73fc84901, hash: 167025b18e584ac0a3e78a398e3d67ba, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 5, date: 2023-06-25T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: bfc1cbdf1a44ff9e3cc2e4343cd83f36, hash: 0af3c96097a94404d0e03a9d773b89e2, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 6, date: 2023-06-26T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 389e72a009ffec91036ce49a8a3685ad, hash: 3c04aec004104974e01e773c79ff7ad9, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 7, date: 2023-06-27T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 7b2b6554818a402aef8b285d969663f1, hash: 49b22ab6793de4ba0b3fb13df91b38e6, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 8, date: 2023-06-28T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: e6b18e4fa1625d4e90c3a200e92f8f3f, hash: ad2d74d25c9da6b271a924908047659f, chainID: 090bf2f8b55be45b4661b24b7e9c340c, seq: 9, date: 2023-06-29T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
        ];

        let expected_bytewords = [
            "axis bald whiz yoga race help vibe help frog huts purr gear knob news edge barn inky jump mild warm warm pose obey ruby very hill yank song frog into maze work into game zero mild fish plus fact keep heat jowl loud dark horn navy sets heat menu peck fern fizz item cost rich mild nail omit drum meow sets work zinc edge beta zero draw redo king math real dice onyx cats help iris join oboe nail slot barn news jury fact miss eyes love fair yawn frog vast",
            "beta ruby film fund kept gems limp runs edge visa twin hawk brag calm keys list lion iron luck puff huts city loud exam peck song tuna quad vibe void wall road sets drum drum gush oboe luck liar solo gala zaps onyx gear warm hang road luck data epic logo days what huts work main echo wolf exam peck fuel song easy holy real taxi zinc soap curl aqua jury rust cusp slot bulb plus film lamb jazz cusp limp zone zone lion jade item surf claw soap ruby code",
            "vibe zinc hill bias webs open drum rust peck soap body iced zoom exit fern zaps beta frog hill pool away fact frog dice aunt huts time girl paid taxi paid void body acid junk tied tomb main noon monk fuel tomb plus days epic hope obey vast open monk purr jowl luck numb diet time surf claw luau kiln safe redo diet jazz belt vows what code undo whiz guru epic fern bulb lung monk loud many claw claw gift gems door slot note cola atom visa epic main down",
            "roof body many zest brag curl frog drop blue grim leaf down wall limp purr sets join item deli figs away zone poem when idle frog jolt play next crux hang fact what lazy toys road fern tiny wave gala safe wand urge king help numb vibe cook aunt rust noon guru iron claw hill also drop view beta waxy love down item horn hang dull love free fair paid time gyro meow epic cola zero tiny tied pool list list gear slot gyro need gems vast days safe mild redo",
            "skew slot hill yank part rich road play meow down hope obey cost race purr limp apex note buzz belt echo skew hawk trip junk days keep memo half swan hang cusp gear view junk junk ugly ugly user flap love chef drum gems diet cost poem list king zero wasp free view whiz arch road acid also city mint flap safe lung chef fund good paid jump aunt jazz dull fern toys edge cash fizz keno hope fish owls toil ramp duty axis vast unit wall good cusp visa silk",
            "need data twin owls idle horn idea owls cats figs yell toys fish soap gala acid tuna hang hill game zoom king owls king oboe news safe runs monk gift blue idle holy pose hope pose gift yawn fact whiz user atom puff tiny buzz aqua mild guru film guru undo gift ruin item wall open body barn limp news veto tuna mint join into into huts stub zone tiny waxy fish each city swan tuna crux jugs data road liar wasp fair open tiny swan eyes gray idle exam also",
            "runs safe stub user city foxy zoom noon fern saga vibe edge fern trip fish even pool exit liar purr fish luau figs cook legs time also apex into cash nail item poem hope void ruby veto aqua fact saga draw gala many oboe stub lazy jowl cash junk yoga acid saga poem into cyan omit each wolf away zoom inky film runs urge dice numb gift even kiwi obey surf ramp meow play fern rust roof drop cusp dice limp axis into chef zone iced kick each knob paid heat",
            "exit noon jump numb axis zoom wasp maze apex jazz vibe navy love even limp poem open beta logo slot paid chef join poem city quiz away whiz drop barn claw jugs chef keno zinc claw saga warm wasp scar mint limp hang easy trip purr wolf noon echo runs navy iron item quad navy exit junk navy jolt flew item guru iron vibe noon wand fund navy join half ugly part code warm jazz free item eyes grim hope wolf hard lava user omit obey owls tent need gray wall",
            "king down inch gush lazy love fizz door webs luau dice hill mint mint idea when gems door safe need visa inch guru noon legs omit monk unit back crux whiz days liar drop monk yawn math ramp iris oboe kept gray bias trip able fair eyes fuel obey iris gray dark flux paid gear kite arch silk flap tied huts rust vast aqua toys race trip vast taxi each task echo fair cash cost ugly note drum news poem runs wasp keep cusp data epic open many zone iron brew",
            "visa puma main glow obey iced hill girl math scar oboe able wall dull many fish puma yurt fact jazz pose plus visa keep fish iris wand tiny runs brew brew brag flux lung waxy axis crux legs iced lion slot cats numb task time paid able jury solo bald limp luau surf epic calm hope blue obey echo solo road jolt miss quiz urge yank yell cusp taco draw numb data city heat vial surf urge idle note blue edge jolt buzz horn jugs keno bulb navy yell vows pose",
        ];

        let expected_id_words = [
            "CITY POSE LIST HOPE",
            "GAME RUIN VAST ATOM",
            "LAZY ARCH NUMB MEOW",
            "VERY JUGS WHEN WAXY",
            "TIED KILN JUMP MINT",
            "CALM JUDO DATA PUMA",
            "BACK WOLF SOLO HORN",
            "FERN AQUA POOL RUST",
            "GALA PURR DOOR RAMP",
            "POEM DROP JURY TIED",
        ];

        let expected_bytemoji_ids = [
            "🥳 📡 💬 🍤",
            "🥑 🎷 🐨 😎",
            "💔 😋 🚪 🚨",
            "🐯 🌊 🐞 🐛",
            "👠 🌙 💧 🚁",
            "🤢 💨 👽 💰",
            "🫠 🐺 👖 🍚",
            "🦶 🙄 ⏳ 🏀",
            "🍆 🧲 🙀 🎉",
            "⏰ 🤲 🌀 👠",
        ];

        let expected_urs = [
            "ur:provenance/lfaohdhgasbdwzyarehpvehpfghsprgrkbnseebniyjpmdwmwmpeoyryvyhlyksgfgiomewkiogezomdfhpsftkphtjllddkhnnysshtmupkfnfzimctrhmdnlotdmmwsswkzceebazodwrokgmhrldeoxcshpisjnoenlstbnnsjyftmseslefhqzcsls",
            "ur:provenance/lfaohdhgbaryfmfdktgslprseevatnhkbgcmksltlninlkpfhscyldempksgtaqdvevdwlrdssdmdmghoelklrsogazsoxgrwmhgrdlkdaeclodswthswkmneowfempkflsgeyhyrltizcspclaajyrtcpstbbpsfmlbjzcplpzezelnjeimsfctlevllb",
            "ur:provenance/lfaohdhgvezchlbswsondmrtpkspbyidzmetfnzsbafghlplayftfgdeathsteglpdtipdvdbyadjktdtbmnnnmkfltbpsdsecheoyvtonmkprjllknbdttesfcwluknserodtjzbtvswtceuowzguecfnbblgmkldmycwcwgtgsdrstnecaamvokttifd",
            "ur:provenance/lfaohdhgrfbymyztbgclfgdpbegmlfdnwllpprssjnimdifsayzepmwniefgjtpyntcxhgftwtlytsrdfntywegasewduekghpnbveckatrtnnguincwhlaodpvwbawylednimhnhgdllefefrpdtegomweccazotytdplltltgrstgondgsvtcplssbuy",
            "ur:provenance/lfaohdhgswsthlykptrhrdpymwdnheoyctreprlpaxnebzbteoswhktpjkdskpmohfsnhgcpgrvwjkjkuyuyurfplecfdmgsdtctpmltkgzowpfevwwzahrdadaocymtfpselgcffdgdpdjpatjzdlfntseechfzkohefhostlrpdyasvtutwlghhnrool",
            "ur:provenance/lfaohdhgnddatnosiehniaoscsfsyltsfhspgaadtahghlgezmkgoskgoenssersmkgtbeiehypehepegtynftwzurampftybzaamdgufmguuogtrnimwlonbybnlpnsvotamtjnioiohssbzetywyfhehcysntacxjsdardlrwpfrontysnesgodsinhs",
            "ur:provenance/lfaohdhgrssesburcyfyzmnnfnsaveeefntpfhenpletlrprfhlufscklsteaoaxiochnlimpmhevdryvoaaftsadwgamyoesblyjlchjkyaadsapmiocnotehwfayzmiyfmrsuedenbgtenkioysfrpmwpyfnrtrfdpcpdelpasiocfzeidkkecfnynes",
            "ur:provenance/lfaohdhgetnnjpnbaszmwpmeaxjzvenyleenlppmonbalostpdcfjnpmcyqzaywzdpbncwjscfkozccwsawmwpsrmtlphgeytpprwfnneorsnyinimqdnyetjknyjtfwimguinvennwdfdnyjnhfuyptcewmjzfeimesgmhewfhdlaurotoyostltabsle",
            "ur:provenance/lfaohdhgkgdnihghlylefzdrwsludehlmtmtiawngsdrsendvaihgunnlsotmkutbkcxwzdslrdpmkynmhrpisoektgybstpaefresfloyisgydkfxpdgrkeahskfptdhsrtvtaatsretpvttiehtkeofrchctuynedmnspmrswpkpcpdaeconlurfemjo",
            "ur:provenance/lfaohdhgvapamngwoyidhlglmhsroeaewldlmyfhpaytftjzpepsvakpfhiswdtyrsbwbwbgfxlgwyascxlsidlnstcsnbtktepdaejysobdlplusfeccmhebeoyeosordjtmsqzueykylcptodwnbdacyhtvlsfueienebeeejtbzhnjskobbnnrerpsf",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgasbdwzyarehpvehpfghsprgrkbnseebniyjpmdwmwmpeoyryvyhlyksgfgiomewkiogezomdfhpsftkphtjllddkhnnysshtmupkfnfzimctrhmdnlotdmmwsswkzceebazodwrokgmhrldeoxcshpisjnoenlstbnnsjyftmseslemsgtgybn",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgbaryfmfdktgslprseevatnhkbgcmksltlninlkpfhscyldempksgtaqdvevdwlrdssdmdmghoelklrsogazsoxgrwmhgrdlkdaeclodswthswkmneowfempkflsgeyhyrltizcspclaajyrtcpstbbpsfmlbjzcplpzezelnjeimsfrljkpkwt",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgvezchlbswsondmrtpkspbyidzmetfnzsbafghlplayftfgdeathsteglpdtipdvdbyadjktdtbmnnnmkfltbpsdsecheoyvtonmkprjllknbdttesfcwluknserodtjzbtvswtceuowzguecfnbblgmkldmycwcwgtgsdrstnecaamgemnnlst",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgrfbymyztbgclfgdpbegmlfdnwllpprssjnimdifsayzepmwniefgjtpyntcxhgftwtlytsrdfntywegasewduekghpnbveckatrtnnguincwhlaodpvwbawylednimhnhgdllefefrpdtegomweccazotytdplltltgrstgondgsvtleknlfgh",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgswsthlykptrhrdpymwdnheoyctreprlpaxnebzbteoswhktpjkdskpmohfsnhgcpgrvwjkjkuyuyurfplecfdmgsdtctpmltkgzowpfevwwzahrdadaocymtfpselgcffdgdpdjpatjzdlfntseechfzkohefhostlrpdyasvtutwlztnlwndt",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgnddatnosiehniaoscsfsyltsfhspgaadtahghlgezmkgoskgoenssersmkgtbeiehypehepegtynftwzurampftybzaamdgufmguuogtrnimwlonbybnlpnsvotamtjnioiohssbzetywyfhehcysntacxjsdardlrwpfrontysneszcurcxwy",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgrssesburcyfyzmnnfnsaveeefntpfhenpletlrprfhlufscklsteaoaxiochnlimpmhevdryvoaaftsadwgamyoesblyjlchjkyaadsapmiocnotehwfayzmiyfmrsuedenbgtenkioysfrpmwpyfnrtrfdpcpdelpasiocfzeidkkntskrsrp",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgetnnjpnbaszmwpmeaxjzvenyleenlppmonbalostpdcfjnpmcyqzaywzdpbncwjscfkozccwsawmwpsrmtlphgeytpprwfnneorsnyinimqdnyetjknyjtfwimguinvennwdfdnyjnhfuyptcewmjzfeimesgmhewfhdlaurotoyoskicxfgah",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgkgdnihghlylefzdrwsludehlmtmtiawngsdrsendvaihgunnlsotmkutbkcxwzdslrdpmkynmhrpisoektgybstpaefresfloyisgydkfxpdgrkeahskfptdhsrtvtaatsretpvttiehtkeofrchctuynedmnspmrswpkpcpdaeconcnfekbzm",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgvapamngwoyidhlglmhsroeaewldlmyfhpaytftjzpepsvakpfhiswdtyrsbwbwbgfxlgwyascxlsidlnstcsnbtktepdaejysobdlplusfeccmhebeoyeosordjtmsqzueykylcptodwnbdacyhtvlsfueienebeeejtbzhnjskobbengszmfx",
        ];

        run_test(
            ProvenanceMarkResolution::Quartile,
            true,
            &expected_display,
            &expected_debug,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
        );
    }

    #[test]
    fn test_high() {
        let expected_display = [
            "ProvenanceMark(5a9b5c46)",
            "ProvenanceMark(b6fbcd03)",
            "ProvenanceMark(7a4984dc)",
            "ProvenanceMark(50a96b79)",
            "ProvenanceMark(c800ae46)",
            "ProvenanceMark(f448fa07)",
            "ProvenanceMark(c5f98082)",
            "ProvenanceMark(acb93cf3)",
            "ProvenanceMark(ece96664)",
            "ProvenanceMark(ef01babb)",
        ];

        let expected_debug = [
            r#"ProvenanceMark(key: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, hash: 5a9b5c46c5156c4549c7c8fc6bf578a97e0a64d3177c7abf3bcbfe52039c9ff8, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 0, date: 2023-06-20T12:00:00Z)"#,
            r#"ProvenanceMark(key: 0ebd3e48774c85bf34e6da5912167887e4fd5d0fefa52ec0aac81162ff383cfa, hash: b6fbcd034beb0c7a965db946ec87e4abaf81fd4e1716f7971eb77ac670a3ab40, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 1, date: 2023-06-21T12:00:00Z)"#,
            r#"ProvenanceMark(key: bc118ffc1221462d1052822be985b2c4c6c75df5a9b9baab942b5fa11fb5b285, hash: 7a4984dc7cbd5df45902e505ca386f756467d86effea30f5636f8ef01a070380, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 2, date: 2023-06-22T12:00:00Z)"#,
            r#"ProvenanceMark(key: 9b25daa7646063a7183df7d73fc84901bfc1cbdf1a44ff9e3cc2e4343cd83f36, hash: 50a96b79a1db3802224f74e6c999d6e43e8711bb3adcbcdc48805a5e825a2c48, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 3, date: 2023-06-23T12:00:00Z)"#,
            r#"ProvenanceMark(key: 389e72a009ffec91036ce49a8a3685ad7b2b6554818a402aef8b285d969663f1, hash: c800ae463162be33320ab116286d40f86366893814dd1da06bfe584e8368cee8, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 4, date: 2023-06-24T12:00:00Z)"#,
            r#"ProvenanceMark(key: e6b18e4fa1625d4e90c3a200e92f8f3fc94422e7b185b885d6b6ae36c053d957, hash: f448fa0702d6d57c85ba1f7c796d6f954cf802f9c5f9972de3c8ca2a76baebd9, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 5, date: 2023-06-25T12:00:00Z)"#,
            r#"ProvenanceMark(key: 46af21cf196e347ba26781dd568269286e1d13d819b54eb915daa5bde40f77a6, hash: c5f98082d69a4bef3fdfd8137e315a0b7f70df2ec116c621dadc9a4988b6a5cf, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 6, date: 2023-06-26T12:00:00Z)"#,
            r#"ProvenanceMark(key: 8d935aad15e82ccaf5a070450e3528745858cf689e80ad334e36d00d7c86b365, hash: acb93cf34777d16ed22dcd8685ac04d8c8962ae3af081496aecd41a6720100a0, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 7, date: 2023-06-27T12:00:00Z)"#,
            r#"ProvenanceMark(key: e9da846ee810ce4834ac8010928059e40662b7fcec647e209aa6aed1da22c302, hash: ece966643b274a361b64c230edd09b8d7045fb498051346fc868c6fbaa5df77a, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 8, date: 2023-06-28T12:00:00Z)"#,
            r#"ProvenanceMark(key: adbcef759c8a0b70a84eeb6d971d9bdecbb3288ecc0f1fb47f7d613f912e38bc, hash: ef01babb78eaab6fdfe463749dfc3d25ae1289e9f0566714797175f87949c879, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 9, date: 2023-06-29T12:00:00Z)"#,
        ];

        let expected_bytewords = [
            "axis bald whiz yoga race help vibe help frog huts purr gear knob news edge barn yurt frog gems hope wall high liar yank lava mild game peck ruin away holy kite open fizz dark data lava brew gyro curl glow eyes body ramp epic tiny high item join navy solo gala away view acid real ruin open keys body dice stub fair inch pose poem hang logo list luck cash kiwi logo tiny twin oval help vows chef safe skew half exam kiln cash news omit cusp fern vial into very huts visa quiz curl hang bald menu draw curl noon jolt game legs void game drop iris calm",
            "beta ruby film fund kept gems limp runs edge visa twin hawk brag calm keys list vibe zinc hill bias webs open drum rust peck soap body iced zoom exit fern zaps yurt quiz obey swan hard film lamb numb kept bulb idea ruin limp cost junk skew peck waxy lava wolf love grim zest foxy veto vibe maze junk foxy stub days void wand lava lamb hill when help part mild flap ramp door able cusp need game flap kept nail zero zaps good luck kiwi free safe unit fuel maze lazy join brag back nail vial time navy keys lamb into play obey flap holy wave unit slot",
            "roof body many zest brag curl frog drop blue grim leaf down wall limp purr sets skew slot hill yank part rich road play meow down hope obey cost race purr limp lion toys rust horn slot wall even waxy ramp zaps rust jazz obey jolt item easy list gear drop song jowl diet quad sets cyan zest dark jade lazy gear lion menu dark judo onyx race girl horn hope main limp free dice blue trip jazz vial acid mild yank toys down miss good limp atom acid zinc unit keep jowl surf axis easy tiny echo flux open eyes leaf cats math tent down ruby eyes wave claw",
            "need data twin owls idle horn idea owls cats figs yell toys fish soap gala acid runs safe stub user city foxy zoom noon fern saga vibe edge fern trip fish even flap knob free task next iron scar need meow keys kiln wall duty scar wolf yell draw fuel pool calm kick runs memo guru kiln real cost keys cats jade obey half fund omit cash able math deli ramp taco yell kiln vast diet cost zero runs stub jade into taco need days yell easy memo claw exam limp oval scar slot yawn iris love rock glow lava fact need belt bulb stub vast dice wand list slot",
            "exit noon jump numb axis zoom wasp maze apex jazz vibe navy love even limp poem king down inch gush lazy love fizz door webs luau dice hill mint mint idea when jump race urge rust stub figs lava cost stub zoom item poem idle epic buzz whiz barn redo hang scar skew warm calm quad poem zest vibe vows when toil many poem good monk judo fact wasp jazz twin away fair monk data nail eyes calm eyes keno dull veto taxi love iron quad saga meow tied easy item zaps kite veto oval maze open love sets drop navy waxy quad play even dark nail tuna legs iced",
            "visa puma main glow obey iced hill girl math scar oboe able wall dull many fish solo foxy cusp void puma limp redo limp tomb ramp pool even rust guru tuna hang zaps cola part film brag able pool road slot ramp draw bulb diet luau horn good away figs away film maze list hope zero vibe liar very belt door tied gear aunt vibe inch fund back quad jade menu help guru when mild lion love tiny cook dark back quad frog onyx fund yell dice vows hang onyx hope miss cyan hope brag buzz wolf dull rock iron oboe game yurt main purr mint game plus list tomb",
            "frog pose curl task chef jolt edge king oboe into lazy unit half leaf iron dice jolt cola brew trip chef race girl rich buzz twin open ruby vibe bias kept oval fern yoga kept ruby brag half blue rich math what paid zero luck flux wall holy knob plus iron back lion curl toys plus kick onyx diet menu bias film memo curl figs fuel zero oboe surf lung note code good inch yell solo each aunt cats tomb free yurt film part heat inch hang navy part gush dice ruin yawn half part code diet good undo belt kite race half stub mint taxi cusp cola huts luau",
            "lung menu heat poem buzz vows draw song yank numb judo free beta epic dice jury hard hard task iris noon lava poem echo girl even taxi belt kite lion quad inch huts task quiz chef cusp cola judo loud tuna skew aunt yurt flux miss inch navy belt news door yoga tied visa city wall help task stub fizz kite poem silk acid sets hard edge pose code dice flap memo whiz able user oboe leaf quiz mint guru need undo scar twin rust jazz swan exam bias redo idle need edge silk lung zero kite jowl wand waxy brag dark undo song kiwi tent cats toil miss junk",
            "wall twin liar jolt vows blue taco fund edge plus lava blue memo lava hawk vibe atom iced real zest wasp idle knob crux navy oval pool tent twin cusp scar also memo mint fish warm main cats yell back atom tomb wall void cola cats memo quiz deli oval purr sets yell monk blue gyro zinc body toys owls quad keno cusp toil ugly grim fern each vast heat slot obey vows saga need play lava keno liar kick webs keys gems jury flew junk soap plus lava wolf exam claw veto slot barn dice blue iris solo list main jazz surf pose fish gyro runs days body fizz",
            "poem roof webs keep news love bald judo paid girl warm join miss cola need urge stub quad dice main surf bias cost quiz lamb kiwi huts fish maze drum exit roof song cook knob game road numb exit huts cusp cola mint redo glow rock main purr work yoga wasp urge miss kept lamb cyan plus lava dull twin need fish holy many iron foxy webs wolf axis list zinc omit aunt fair lava wolf cusp able pose data city cook glow redo work iced paid wave easy monk gala chef road idle jump cola inch days void ramp vows warm days film quad wave poem acid tomb axis",
        ];

        let expected_id_words = [
            "HEAT NEED HIGH FROG",
            "RAMP ZERO SWAN APEX",
            "KILN GALA LIAR UNDO",
            "GOOD PART JADE KICK",
            "SOAP ABLE POOL FROG",
            "WORK FUND ZAPS AUNT",
            "SILK YURT LAVA LEAF",
            "PLUS RICH FERN WOLF",
            "WASP WALL INKY IDLE",
            "WEBS ACID ROAD ROCK",
        ];

        let expected_bytemoji_ids = [
            "🍕 🎢 🥙 🍑",
            "🎉 🐚 🧢 😉",
            "🌙 🍆 💕 🐰",
            "🧄 🧮 🌹 🌜",
            "👚 😀 ⏳ 🍑",
            "🐍 🥝 🦀 😍",
            "🔥 🦞 💛 💘",
            "📷 🫖 🦶 🐺",
            "🦄 🦆 🌵 🎂",
            "🦋 😂 🔭 🛁",
        ];

        let expected_urs = [
            "ur:provenance/lfaxhdimasbdwzyarehpvehpfghsprgrkbnseebnytfggshewlhhlryklamdgepkrnayhykeonfzdkdalabwgoclgwesbyrpectyhhimjnnysogaayvwadrlrnonksbydesbfrihpepmhgloltlkchkilotytnolhpvscfseswhfemknchnsotcpfnvliovyhsvaqzclhgbdmudwclnnjtgelsvdhpfmclbw",
            "ur:provenance/lfaxhdimbaryfmfdktgslprseevatnhkbgcmksltvezchlbswsondmrtpkspbyidzmetfnzsytqzoysnhdfmlbnbktbbiarnlpctjkswpkwylawflegmztfyvovemejkfysbdsvdwdlalbhlwnhpptmdfprpdraecpndgefpktnlzozsgdlkkifeseutflmelyjnbgbknlvltenykslbiopyoyfpgwzemwsa",
            "ur:provenance/lfaxhdimrfbymyztbgclfgdpbegmlfdnwllpprssswsthlykptrhrdpymwdnheoyctreprlplntsrthnstwlenwyrpzsrtjzoyjtimeyltgrdpsgjldtqdsscnztdkjelygrlnmudkjooxreglhnhemnlpfedebetpjzvladmdyktsdnmsgdlpamadzcutkpjlsfaseytyeofxoneslfcsmhttdnpsdroxck",
            "ur:provenance/lfaxhdimnddatnosiehniaoscsfsyltsfhspgaadrssesburcyfyzmnnfnsaveeefntpfhenfpkbfetkntinsrndmwksknwldysrwfyldwflplcmkkrsmoguknrlctkscsjeoyhffdotchaemhdirptoylknvtdtctzorssbjeiotonddsyleymocwemlpolsrstynislerkgwlaftndbtbbsbvtesyttosa",
            "ur:provenance/lfaxhdimetnnjpnbaszmwpmeaxjzvenyleenlppmkgdnihghlylefzdrwsludehlmtmtiawnjpreuertsbfslactsbzmimpmieecbzwzbnrohgsrswwmcmqdpmztvevswntlmypmgdmkjoftwpjztnayfrmkdanlescmeskodlvotileinqdsamwtdeyimzskevoolmeonlessdpnywyqdpyendklosgsgio",
            "ur:provenance/lfaxhdimvapamngwoyidhlglmhsroeaewldlmyfhsofycpvdpalprolptbrpplenrtgutahgzscaptfmbgaeplrdstrpdwbbdtluhngdayfsayfmmelthezovelrvybtdrtdgratveihfdbkqdjemuhpguwnmdlnletyckdkbkqdfgoxfdyldevshgoxhemscnhebgbzwfdlrkinoegeytmnprmthprstote",
            "ur:provenance/lfaxhdimfgpecltkcfjteekgoeiolyuthflfindejtcabwtpcfreglrhbztnonryvebsktolfnyaktrybghfberhmhwtpdzolkfxwlhykbpsinbklncltspskkoxdtmubsfmmoclfsflzooesflgnecegdihylsoehatcstbfeytfmpthtihhgnyptghdernynhfptcedtgduobtkerehfsbmttieobademn",
            "ur:provenance/lfaxhdimlgmuhtpmbzvsdwsgyknbjofebaecdejyhdhdtkisnnlapmeoglentibtkelnqdihhstkqzcfcpcajoldtaswatytfxmsihnybtnsdryatdvacywlhptksbfzkepmskadsshdeepecedefpmowzaeuroelfqzmtgunduosrtnrtjzsnembsroiendeesklgzokejlwdwybgdkuosgkittasswueko",
            "ur:provenance/lfaxhdimwltnlrjtvsbetofdeepslabemolahkveamidrlztwpiekbcxnyolpltttncpsraomomtfhwmmncsylbkamtbwlvdcacsmoqzdiolprssylmkbegozcbytsosqdkocptluygmfnehvthtstoyvssandpylakolrkkwsksgsjyfwjksppslawfemcwvostbndebeissoltmnjzsfpefhgoplechdfe",
            "ur:provenance/lfaxhdimpmrfwskpnslebdjopdglwmjnmscanduesbqddemnsfbsctqzlbkihsfhmedmetrfsgckkbgerdnbethscpcamtrogwrkmnprwkyawpuemsktlbcnpsladltnndfhhymyinfywswfasltzcotatfrlawfcpaepedacyckgwrowkidpdweeymkgacfrdiejpcaihdsvdrpvswmdsfmqdwerfbgnebn",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimasbdwzyarehpvehpfghsprgrkbnseebnytfggshewlhhlryklamdgepkrnayhykeonfzdkdalabwgoclgwesbyrpectyhhimjnnysogaayvwadrlrnonksbydesbfrihpepmhgloltlkchkilotytnolhpvscfseswhfemknchnsotcpfnvliovyhsvaqzclhgbdmudwclnnjtgelsvdflhsmyla",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimbaryfmfdktgslprseevatnhkbgcmksltvezchlbswsondmrtpkspbyidzmetfnzsytqzoysnhdfmlbnbktbbiarnlpctjkswpkwylawflegmztfyvovemejkfysbdsvdwdlalbhlwnhpptmdfprpdraecpndgefpktnlzozsgdlkkifeseutflmelyjnbgbknlvltenykslbiopyoyfpguoyftgy",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimrfbymyztbgclfgdpbegmlfdnwllpprssswsthlykptrhrdpymwdnheoyctreprlplntsrthnstwlenwyrpzsrtjzoyjtimeyltgrdpsgjldtqdsscnztdkjelygrlnmudkjooxreglhnhemnlpfedebetpjzvladmdyktsdnmsgdlpamadzcutkpjlsfaseytyeofxoneslfcsmhttdnpfkpbklg",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimnddatnosiehniaoscsfsyltsfhspgaadrssesburcyfyzmnnfnsaveeefntpfhenfpkbfetkntinsrndmwksknwldysrwfyldwflplcmkkrsmoguknrlctkscsjeoyhffdotchaemhdirptoylknvtdtctzorssbjeiotonddsyleymocwemlpolsrstynislerkgwlaftndbtbbsbvtdaolhngy",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimetnnjpnbaszmwpmeaxjzvenyleenlppmkgdnihghlylefzdrwsludehlmtmtiawnjpreuertsbfslactsbzmimpmieecbzwzbnrohgsrswwmcmqdpmztvevswntlmypmgdmkjoftwpjztnayfrmkdanlescmeskodlvotileinqdsamwtdeyimzskevoolmeonlessdpnywyqdpyendkmwmdiewk",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimvapamngwoyidhlglmhsroeaewldlmyfhsofycpvdpalprolptbrpplenrtgutahgzscaptfmbgaeplrdstrpdwbbdtluhngdayfsayfmmelthezovelrvybtdrtdgratveihfdbkqdjemuhpguwnmdlnletyckdkbkqdfgoxfdyldevshgoxhemscnhebgbzwfdlrkinoegeytmnprmtflvthnfz",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimfgpecltkcfjteekgoeiolyuthflfindejtcabwtpcfreglrhbztnonryvebsktolfnyaktrybghfberhmhwtpdzolkfxwlhykbpsinbklncltspskkoxdtmubsfmmoclfsflzooesflgnecegdihylsoehatcstbfeytfmpthtihhgnyptghdernynhfptcedtgduobtkerehfsbmttidlgylnca",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimlgmuhtpmbzvsdwsgyknbjofebaecdejyhdhdtkisnnlapmeoglentibtkelnqdihhstkqzcfcpcajoldtaswatytfxmsihnybtnsdryatdvacywlhptksbfzkepmskadsshdeepecedefpmowzaeuroelfqzmtgunduosrtnrtjzsnembsroiendeesklgzokejlwdwybgdkuosgkittbznljovw",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimwltnlrjtvsbetofdeepslabemolahkveamidrlztwpiekbcxnyolpltttncpsraomomtfhwmmncsylbkamtbwlvdcacsmoqzdiolprssylmkbegozcbytsosqdkocptluygmfnehvthtstoyvssandpylakolrkkwsksgsjyfwjksppslawfemcwvostbndebeissoltmnjzsfpefhgoprimyntb",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimpmrfwskpnslebdjopdglwmjnmscanduesbqddemnsfbsctqzlbkihsfhmedmetrfsgckkbgerdnbethscpcamtrogwrkmnprwkyawpuemsktlbcnpsladltnndfhhymyinfywswfasltzcotatfrlawfcpaepedacyckgwrowkidpdweeymkgacfrdiejpcaihdsvdrpvswmdsfmqdwenbgtehne",
        ];

        run_test(
            ProvenanceMarkResolution::High,
            false,
            &expected_display,
            &expected_debug,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
        );
    }

    #[test]
    fn test_high_with_info() {
        let expected_display = [
            "ProvenanceMark(cb1344dc)",
            "ProvenanceMark(4a7b870e)",
            "ProvenanceMark(0c6e4cdd)",
            "ProvenanceMark(fb62bb3a)",
            "ProvenanceMark(b75e2a59)",
            "ProvenanceMark(219c5d85)",
            "ProvenanceMark(a18ac2db)",
            "ProvenanceMark(0eb51fcd)",
            "ProvenanceMark(bffa6872)",
            "ProvenanceMark(137e720b)",
        ];

        let expected_debug = [
            r#"ProvenanceMark(key: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, hash: cb1344dce2b6ea1ae0bb3dffbcf71080d8524e07341b44eb8c9b73ba9a4e9642, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 0, date: 2023-06-20T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 0ebd3e48774c85bf34e6da5912167887e4fd5d0fefa52ec0aac81162ff383cfa, hash: 4a7b870e02c344106970a6372f4f999334497fc272e10595acb82aef56b23d21, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 1, date: 2023-06-21T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: bc118ffc1221462d1052822be985b2c4c6c75df5a9b9baab942b5fa11fb5b285, hash: 0c6e4cdd9d4f2cda7d7bebe3b0f1362fafa2aa37de4cd4ec36cd1c4621db2a8f, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 2, date: 2023-06-22T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 9b25daa7646063a7183df7d73fc84901bfc1cbdf1a44ff9e3cc2e4343cd83f36, hash: fb62bb3a7038cb4e3048fa75f6dd844ddc0e3a6635ffa9c416917f51e5b6b4a0, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 3, date: 2023-06-23T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 389e72a009ffec91036ce49a8a3685ad7b2b6554818a402aef8b285d969663f1, hash: b75e2a59f7d4301738f8a973ba3c6c1519b6b0c8b0f9ec865a917903c28a16dd, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 4, date: 2023-06-24T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: e6b18e4fa1625d4e90c3a200e92f8f3fc94422e7b185b885d6b6ae36c053d957, hash: 219c5d852e8aaeb83999a86dc879cfe83a6cc75025e47462649b4a009127e23e, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 5, date: 2023-06-25T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 46af21cf196e347ba26781dd568269286e1d13d819b54eb915daa5bde40f77a6, hash: a18ac2db249959e829d671445cb1799841d32f56ae56913daebf78c5d7b471cf, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 6, date: 2023-06-26T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 8d935aad15e82ccaf5a070450e3528745858cf689e80ad334e36d00d7c86b365, hash: 0eb51fcd44c133d4e2b41354ed2a0bee544dfb932f8ee113f053dd0ca3c1aafc, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 7, date: 2023-06-27T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: e9da846ee810ce4834ac8010928059e40662b7fcec647e209aa6aed1da22c302, hash: bffa68723de66208cc5fa90453d2ea08a0c4b2869a26f3649240c06c245b1fef, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 8, date: 2023-06-28T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: adbcef759c8a0b70a84eeb6d971d9bdecbb3288ecc0f1fb47f7d613f912e38bc, hash: 137e720b0e660fbd4a0f3f16f6af24e7e647bba6788bb266cfc092bcce361846, chainID: 090bf2f8b55be45b4661b24b7e9c340cf9464c5fe95c84f580954aaabe085e7c, seq: 9, date: 2023-06-29T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
        ];

        let expected_bytewords = [
            "axis bald whiz yoga race help vibe help frog huts purr gear knob news edge barn yurt frog gems hope wall high liar yank lava mild game peck ruin away holy kite open fizz dark data lava brew gyro curl glow eyes body ramp epic tiny high item join navy solo gala away view acid real ruin open keys body dice stub fair inch film data glow brag numb dull maze cusp curl paid dull open luck wand jugs vows horn beta cola pool edge zero next keno luau quad wand axis yoga edge ruby need hang bald menu draw curl noon jolt game legs void solo lazy zone unit judo limp yoga luck kick numb undo oboe code edge zone yank time note surf king blue waxy exit item gear holy zest good good zest knob edge taxi",
            "beta ruby film fund kept gems limp runs edge visa twin hawk brag calm keys list vibe zinc hill bias webs open drum rust peck soap body iced zoom exit fern zaps yurt quiz obey swan hard film lamb numb kept bulb idea ruin limp cost junk skew peck waxy lava wolf love grim zest foxy veto vibe maze junk foxy stub days void calm able epic good redo junk very zoom ruin need epic jugs very guru exam kick wasp gray kick keno epic king many fuel junk tied cash redo owls kite liar jade nail vial time navy keys lamb into play obey flap stub paid hill belt redo epic zoom logo cash help wolf deli quiz keep also echo epic race runs dice eyes menu leaf axis jowl bald noon game cost yoga rich tuna liar",
            "roof body many zest brag curl frog drop blue grim leaf down wall limp purr sets skew slot hill yank part rich road play meow down hope obey cost race purr limp lion toys rust horn slot wall even waxy ramp zaps rust jazz obey jolt item easy list gear drop song jowl diet quad sets cyan zest dark jade lazy gear lion menu grim hang jazz quiz pose memo drum numb obey fern days yawn oboe open road help holy duty open jump ramp yawn huts cost gush hope glow scar gush blue crux figs tiny echo flux open eyes leaf cats math tent down barn rich cash fair purr holy unit blue holy deli omit puma flap item next aqua open each real hang jury hill user down yoga fact gems tied limp omit play gear song",
            "need data twin owls idle horn idea owls cats figs yell toys fish soap gala acid runs safe stub user city foxy zoom noon fern saga vibe edge fern trip fish even flap knob free task next iron scar need meow keys kiln wall duty scar wolf yell draw fuel pool calm kick runs memo guru kiln real cost keys cats jade obey half vial iris slot flux flap sets free leaf view kiwi jolt road crux runs wave iced loud waxy view frog diet tiny deli love free days numb part onyx down jolt lava love rock glow lava fact need belt bulb stub vast fact holy iced item lung echo yank fizz rich list jolt gear yoga play inch quad jump blue cook fern note buzz sets race urge beta tent heat brew quad wolf leaf aqua",
            "exit noon jump numb axis zoom wasp maze apex jazz vibe navy love even limp poem king down inch gush lazy love fizz door webs luau dice hill mint mint idea when jump race urge rust stub figs lava cost stub zoom item poem idle epic buzz whiz barn redo hang scar skew warm calm quad poem zest vibe vows when toil many poem dull skew work data door twin gush draw each item figs zest play fuel buzz need gyro easy wall kiln swan miss echo purr vial hill gear real figs able knob onyx open love sets drop navy waxy quad play even dark junk user foxy holy pose main gala time lamb jury gift fair atom urge grim hope peck fish ruby wall next fish void drop grim flux pool lamb yell idea plus wasp peck",
            "visa puma main glow obey iced hill girl math scar oboe able wall dull many fish solo foxy cusp void puma limp redo limp tomb ramp pool even rust guru tuna hang zaps cola part film brag able pool road slot ramp draw bulb diet luau horn good away figs away film maze list hope zero vibe liar very belt door tied gear aunt each puma webs logo note exam vows note webs tied cusp miss fair rust ruin hawk kite deli legs belt paid wand stub owls taxi yell user ruby sets saga claw whiz wolf dull rock iron oboe game yurt main purr mint ruin axis even void iron solo yell noon meow jury half saga axis play data plus fish ramp unit puff drop frog gray dull dark idle onyx code warm tiny guru runs quiz",
            "frog pose curl task chef jolt edge king oboe into lazy unit half leaf iron dice jolt cola brew trip chef race girl rich buzz twin open ruby vibe bias kept oval fern yoga kept ruby brag half blue rich math what paid zero luck flux wall holy knob plus iron back lion curl toys plus kick onyx diet menu bias film memo curl hawk edge rich zero film main lung claw frog jazz holy noon brew list fair free king heat taco tent epic data able lion unit exam song easy part gush kiwi code diet good undo belt kite race half stub mint taxi safe wand runs zinc zest drop love hope navy runs unit buzz what jolt hope drop miss real tuna love song work grim easy omit foxy whiz news task need wand swan iced",
            "lung menu heat poem buzz vows draw song yank numb judo free beta epic dice jury hard hard task iris noon lava poem echo girl even taxi belt kite lion quad inch huts task quiz chef cusp cola judo loud tuna skew aunt yurt flux miss inch navy belt news door yoga tied visa city wall help task stub fizz kite poem silk acid inky gush cash maze cost noon omit dice saga nail acid judo wand easy nail inch aunt aunt brag peck fizz wand exit purr gray days yoga each view arch deli owls kite jowl wand waxy brag dark undo song kiwi tent vial miss solo idle flux owls wall kiln puff eyes down fact lazy surf when kiln holy calm days figs undo numb fair good open kite ugly lava pool silk inch brag runs",
            "wall twin liar jolt vows blue taco fund edge plus lava blue memo lava hawk vibe atom iced real zest wasp idle knob crux navy oval pool tent twin cusp scar also memo mint fish warm main cats yell back atom tomb wall void cola cats memo quiz deli oval purr sets yell monk blue gyro zinc body toys owls quad keno cusp toil logo flap easy deli visa need webs note fish yurt what note film jury yank zest fish yurt arch rock hard aqua bias owls twin ugly each luck jazz safe vibe ruby blue iris solo list main jazz surf pose fish gyro puff wasp even brag blue zone fuel logo loud gush able undo mild plus unit toil oboe away blue poem veto gyro dull horn free lava yell oboe plus blue hope kick many",
            "poem roof webs keep news love bald judo paid girl warm join miss cola need urge stub quad dice main surf bias cost quiz lamb kiwi huts fish maze drum exit roof song cook knob game road numb exit huts cusp cola mint redo glow rock main purr work yoga wasp urge miss kept lamb cyan plus lava dull twin need fish holy many mild fair deli flux lamb bald hawk jugs memo taxi undo maze gala guru ramp void grim gear kiwi yell kite runs kiwi note liar diet pool hill belt claw oboe cusp inch days void ramp vows warm days film quad wave play vial scar math ugly trip heat knob song lion high owls noon love judo sets soap zero numb city high days flap keep door road menu inky dark taco aqua song slot",
        ];

        let expected_id_words = [
            "STUB BREW FOXY UNDO",
            "GAME KING LIST BETA",
            "BARN JOLT GEMS UNIT",
            "ZERO ICED ROCK FACT",
            "REAL HOLY DOOR HAWK",
            "CURL NEWS HILL LIMP",
            "OBEY LOVE SAGA UGLY",
            "BETA RACE COST SWAN",
            "RUNS ZAPS IRIS JUMP",
            "BREW KNOB JUMP BALD",
        ];

        let expected_bytemoji_ids = [
            "👗 🤪 🫐 🐰",
            "🥑 🌎 💬 🤨",
            "🤩 🌻 🍅 🦊",
            "🐚 🍨 🛁 👀",
            "🪭 🍜 🙀 🍟",
            "👹 🎠 🍱 🏁",
            "🪑 🔴 🎾 🐹",
            "🤨 🎀 🤯 🧢",
            "🎺 🦀 💐 💧",
            "🤪 🪐 💧 🥱",
        ];

        let expected_urs = [
            "ur:provenance/lfaxhdltasbdwzyarehpvehpfghsprgrkbnseebnytfggshewlhhlryklamdgepkrnayhykeonfzdkdalabwgoclgwesbyrpectyhhimjnnysogaayvwadrlrnonksbydesbfrihfmdagwbgnbdlmecpclpddlonlkwdjsvshnbacapleezontkoluqdwdasyaeeryndhgbdmudwclnnjtgelsvdsolyzeutjolpyalkkknbuooeceeezeyktenesfkgbewyetimgrhyztgdgdskonahqz",
            "ur:provenance/lfaxhdltbaryfmfdktgslprseevatnhkbgcmksltvezchlbswsondmrtpkspbyidzmetfnzsytqzoysnhdfmlbnbktbbiarnlpctjkswpkwylawflegmztfyvovemejkfysbdsvdcmaeecgdrojkvyzmrnndecjsvyguemkkwpgykkkoeckgmyfljktdchrooskelrjenlvltenykslbiopyoyfpsbpdhlbtroeczmlochhpwfdiqzkpaoeoecrersdeesmulfasjlbdnngectseidvsvt",
            "ur:provenance/lfaxhdltrfbymyztbgclfgdpbegmlfdnwllpprssswsthlykptrhrdpymwdnheoyctreprlplntsrthnstwlenwyrpzsrtjzoyjtimeyltgrdpsgjldtqdsscnztdkjelygrlnmugmhgjzqzpemodmnboyfndsynoeonrdhphydyonjprpynhsctghhegwsrghbecxfstyeofxoneslfcsmhttdnbnrhchfrprhyutbehydiotpafpimntaaonehrlhgjyhlurdnyaftgstdlpnyjoknpl",
            "ur:provenance/lfaxhdltnddatnosiehniaoscsfsyltsfhspgaadrssesburcyfyzmnnfnsaveeefntpfhenfpkbfetkntinsrndmwksknwldysrwfyldwflplcmkkrsmoguknrlctkscsjeoyhfvlisstfxfpssfelfvwkijtrdcxrsweidldwyvwfgdttydilefedsnbptoxdnjtlalerkgwlaftndbtbbsbvtfthyidimlgeoykfzrhltjtgryapyihqdjpbeckfnnebzssreuebatthtbwledeqdhn",
            "ur:provenance/lfaxhdltetnnjpnbaszmwpmeaxjzvenyleenlppmkgdnihghlylefzdrwsludehlmtmtiawnjpreuertsbfslactsbzmimpmieecbzwzbnrohgsrswwmcmqdpmztvevswntlmypmdlswwkdadrtnghdwehimfsztpyflbzndgoeywlknsnmseoprvlhlgrrlfsaekboxonlessdpnywyqdpyendkjkurfyhypemngatelbjygtframuegmhepkfhrywlntfhvddpgmfxpllbylhtktutto",
            "ur:provenance/lfaxhdltvapamngwoyidhlglmhsroeaewldlmyfhsofycpvdpalprolptbrpplenrtgutahgzscaptfmbgaeplrdstrpdwbbdtluhngdayfsayfmmelthezovelrvybtdrtdgratehpawsloneemvsnewstdcpmsfrrtrnhkkedilsbtpdwdsbostiylurrysssacwwzwfdlrkinoegeytmnprmtrnasenvdinsoylnnmwjyhfsaaspydapsfhrputpfdpfggydldkieoxcewmwelomnti",
            "ur:provenance/lfaxhdltfgpecltkcfjteekgoeiolyuthflfindejtcabwtpcfreglrhbztnonryvebsktolfnyaktrybghfberhmhwtpdzolkfxwlhykbpsinbklncltspskkoxdtmubsfmmoclhkeerhzofmmnlgcwfgjzhynnbwltfrfekghttottecdaaelnutemsgeyptghkicedtgduobtkerehfsbmttisewdrszcztdplehenyrsutbzwtjthedpmsrltalesgwkgmeyotfywznstkoeehztam",
            "ur:provenance/lfaxhdltlgmuhtpmbzvsdwsgyknbjofebaecdejyhdhdtkisnnlapmeoglentibtkelnqdihhstkqzcfcpcajoldtaswatytfxmsihnybtnsdryatdvacywlhptksbfzkepmskadiyghchmectnnotdesanladjowdeynlihatatbgpkfzwdetprgydsyaehvwahdioskejlwdwybgdkuosgkittvlmssoiefxoswlknpfesdnftlysfwnknhycmdsfsuonbfrgdonkeuylaplztrncnuy",
            "ur:provenance/lfaxhdltwltnlrjtvsbetofdeepslabemolahkveamidrlztwpiekbcxnyolpltttncpsraomomtfhwmmncsylbkamtbwlvdcacsmoqzdiolprssylmkbegozcbytsosqdkocptllofpeydivandwsnefhytwtnefmjyykztfhytahrkhdaabsostnuyehlkjzseverybeissoltmnjzsfpefhgopfwpenbgbezeflloldghaeuomdpsuttloeaybepmvogodlhnfelayloepsdtlrfdwm",
            "ur:provenance/lfaxhdltpmrfwskpnslebdjopdglwmjnmscanduesbqddemnsfbsctqzlbkihsfhmedmetrfsgckkbgerdnbethscpcamtrogwrkmnprwkyawpuemsktlbcnpsladltnndfhhymymdfrdifxlbbdhkjsmotiuomegagurpvdgmgrkiylkerskinelrdtplhlbtcwoecpihdsvdrpvswmdsfmqdwepyvlsrmhuytphtkbsglnhhosnnlejossspzonbcyhhdsfpkpdrrdmuiydkylurzoot",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltasbdwzyarehpvehpfghsprgrkbnseebnytfggshewlhhlryklamdgepkrnayhykeonfzdkdalabwgoclgwesbyrpectyhhimjnnysogaayvwadrlrnonksbydesbfrihfmdagwbgnbdlmecpclpddlonlkwdjsvshnbacapleezontkoluqdwdasyaeeryndhgbdmudwclnnjtgelsvdsolyzeutjolpyalkkknbuooeceeezeyktenesfkgbewyetimgrhyztgdgdtbftdaot",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltbaryfmfdktgslprseevatnhkbgcmksltvezchlbswsondmrtpkspbyidzmetfnzsytqzoysnhdfmlbnbktbbiarnlpctjkswpkwylawflegmztfyvovemejkfysbdsvdcmaeecgdrojkvyzmrnndecjsvyguemkkwpgykkkoeckgmyfljktdchrooskelrjenlvltenykslbiopyoyfpsbpdhlbtroeczmlochhpwfdiqzkpaoeoecrersdeesmulfasjlbdnngecttdzcspyl",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltrfbymyztbgclfgdpbegmlfdnwllpprssswsthlykptrhrdpymwdnheoyctreprlplntsrthnstwlenwyrpzsrtjzoyjtimeyltgrdpsgjldtqdsscnztdkjelygrlnmugmhgjzqzpemodmnboyfndsynoeonrdhphydyonjprpynhsctghhegwsrghbecxfstyeofxoneslfcsmhttdnbnrhchfrprhyutbehydiotpafpimntaaonehrlhgjyhlurdnyaftgstdlpldwshtrh",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltnddatnosiehniaoscsfsyltsfhspgaadrssesburcyfyzmnnfnsaveeefntpfhenfpkbfetkntinsrndmwksknwldysrwfyldwflplcmkkrsmoguknrlctkscsjeoyhfvlisstfxfpssfelfvwkijtrdcxrsweidldwyvwfgdttydilefedsnbptoxdnjtlalerkgwlaftndbtbbsbvtfthyidimlgeoykfzrhltjtgryapyihqdjpbeckfnnebzssreuebatthtbwnlrlmukt",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltetnnjpnbaszmwpmeaxjzvenyleenlppmkgdnihghlylefzdrwsludehlmtmtiawnjpreuertsbfslactsbzmimpmieecbzwzbnrohgsrswwmcmqdpmztvevswntlmypmdlswwkdadrtnghdwehimfsztpyflbzndgoeywlknsnmseoprvlhlgrrlfsaekboxonlessdpnywyqdpyendkjkurfyhypemngatelbjygtframuegmhepkfhrywlntfhvddpgmfxpllbylgavszcta",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltvapamngwoyidhlglmhsroeaewldlmyfhsofycpvdpalprolptbrpplenrtgutahgzscaptfmbgaeplrdstrpdwbbdtluhngdayfsayfmmelthezovelrvybtdrtdgratehpawsloneemvsnewstdcpmsfrrtrnhkkedilsbtpdwdsbostiylurrysssacwwzwfdlrkinoegeytmnprmtrnasenvdinsoylnnmwjyhfsaaspydapsfhrputpfdpfggydldkieoxcewmzechplst",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltfgpecltkcfjteekgoeiolyuthflfindejtcabwtpcfreglrhbztnonryvebsktolfnyaktrybghfberhmhwtpdzolkfxwlhykbpsinbklncltspskkoxdtmubsfmmoclhkeerhzofmmnlgcwfgjzhynnbwltfrfekghttottecdaaelnutemsgeyptghkicedtgduobtkerehfsbmttisewdrszcztdplehenyrsutbzwtjthedpmsrltalesgwkgmeyotfywznstkpapluoby",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltlgmuhtpmbzvsdwsgyknbjofebaecdejyhdhdtkisnnlapmeoglentibtkelnqdihhstkqzcfcpcajoldtaswatytfxmsihnybtnsdryatdvacywlhptksbfzkepmskadiyghchmectnnotdesanladjowdeynlihatatbgpkfzwdetprgydsyaehvwahdioskejlwdwybgdkuosgkittvlmssoiefxoswlknpfesdnftlysfwnknhycmdsfsuonbfrgdonkeuylaplwsclaxsf",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltwltnlrjtvsbetofdeepslabemolahkveamidrlztwpiekbcxnyolpltttncpsraomomtfhwmmncsylbkamtbwlvdcacsmoqzdiolprssylmkbegozcbytsosqdkocptllofpeydivandwsnefhytwtnefmjyykztfhytahrkhdaabsostnuyehlkjzseverybeissoltmnjzsfpefhgopfwpenbgbezeflloldghaeuomdpsuttloeaybepmvogodlhnfelayloepsftcwiszt",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltpmrfwskpnslebdjopdglwmjnmscanduesbqddemnsfbsctqzlbkihsfhmedmetrfsgckkbgerdnbethscpcamtrogwrkmnprwkyawpuemsktlbcnpsladltnndfhhymymdfrdifxlbbdhkjsmotiuomegagurpvdgmgrkiylkerskinelrdtplhlbtcwoecpihdsvdrpvswmdsfmqdwepyvlsrmhuytphtkbsglnhhosnnlejossspzonbcyhhdsfpkpdrrdmuiydkvefzuyqz",
        ];

        run_test(
            ProvenanceMarkResolution::High,
            true,
            &expected_display,
            &expected_debug,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
        );
    }

    #[test]
    fn test_readme_deps() {
        version_sync::assert_markdown_deps_updated!("README.md");
    }

    #[test]
    fn test_html_root_url() {
        version_sync::assert_html_root_url_updated!("src/lib.rs");
    }
}
