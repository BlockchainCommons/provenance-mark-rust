mod resolution;
pub use resolution::*;
mod mark;
pub use mark::*;
mod generator;
pub use generator::*;
mod seed;
pub use seed::*;
mod rng_state;
pub use rng_state::*;
mod date;
mod crypto_utils;
mod xoshiro256starstar;
mod util;
mod envelope;

#[cfg(test)]
mod tests {
    use bc_ur::prelude::*;
    use chrono::TimeZone;
    use dcbor::Date;
    use bc_envelope::prelude::*;

    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn run_test(
        resolution: ProvenanceMarkResolution,
        include_info: bool,
        expected_descriptions: &[&str],
        expected_bytewords: &[&str],
        expected_id_words: &[&str],
        expected_bytemoji_ids: &[&str],
        expected_urs: &[&str],
        expected_urls: &[&str],
        only_print: bool
    ) {
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

        if only_print {
            marks.iter().for_each(|mark| println!("{}", mark));
        } else {
            assert_eq!(
                marks
                    .iter()
                    .map(|mark| mark.to_string())
                    .collect::<Vec<_>>(),
                expected_descriptions
            );
        }

        let bytewords = marks
            .iter()
            .map(|mark| mark.to_bytewords())
            .collect::<Vec<_>>();
        if only_print {
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
        if only_print {
            id_words.iter().for_each(|id_word| println!("{:?}", id_word));
        } else {
            assert_eq!(id_words, expected_id_words);
        }

        let bytemoji_ids = marks
            .iter()
            .map(|mark| mark.bytemoji_identifier(false))
            .collect::<Vec<_>>();
        if only_print {
            bytemoji_ids.iter().for_each(|bytemoji_id| println!("{:?}", bytemoji_id));
        } else {
            assert_eq!(bytemoji_ids, expected_bytemoji_ids);
        }

        let urs = marks
            .iter()
            .map(|mark| mark.ur_string())
            .collect::<Vec<_>>();
        if only_print {
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
        if only_print {
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

        for mark in marks {
            let envelope = mark.into_envelope();
            println!("{}", envelope.format());
        }
    }

    #[test]
    fn test_low() {
        let expected_descriptions = [
            r#"ProvenanceMark(key: ce7c1599, hash: 65a4dfbf, chainID: ce7c1599, seq: 0, date: 2023-06-20T00:00:00Z)"#,
            r#"ProvenanceMark(key: 695dafa1, hash: 4f1215da, chainID: ce7c1599, seq: 1, date: 2023-06-21T00:00:00Z)"#,
            r#"ProvenanceMark(key: 38cfe538, hash: 69fd4e2b, chainID: ce7c1599, seq: 2, date: 2023-06-22T00:00:00Z)"#,
            r#"ProvenanceMark(key: bedba2c8, hash: 8b358624, chainID: ce7c1599, seq: 3, date: 2023-06-23T00:00:00Z)"#,
            r#"ProvenanceMark(key: a96ec2da, hash: 3c767e36, chainID: ce7c1599, seq: 4, date: 2023-06-24T00:00:00Z)"#,
            r#"ProvenanceMark(key: d0703671, hash: 5003be92, chainID: ce7c1599, seq: 5, date: 2023-06-25T00:00:00Z)"#,
            r#"ProvenanceMark(key: 19cd0a02, hash: 185b6dc7, chainID: ce7c1599, seq: 6, date: 2023-06-26T00:00:00Z)"#,
            r#"ProvenanceMark(key: 55864d59, hash: 6af32a44, chainID: ce7c1599, seq: 7, date: 2023-06-27T00:00:00Z)"#,
            r#"ProvenanceMark(key: c695d857, hash: 3b6f4a25, chainID: ce7c1599, seq: 8, date: 2023-06-28T00:00:00Z)"#,
            r#"ProvenanceMark(key: d351f7df, hash: c183bf5f, chainID: ce7c1599, seq: 9, date: 2023-06-29T00:00:00Z)"#,
        ];

        let expected_bytewords = [
            "taco kite buzz nail arch fact bias nail apex plus deli wave cats webs ruin legs quiz draw work onyx",
            "iron hill pose obey figs fern brag paid silk leaf blue task maze tomb hang cost solo math menu race",
            "exit task view exit slot jump redo keep bald kick inky kick code door task code quiz toys data edge",
            "ruin ugly oboe soap scar open aqua owls also inch user meow pool love apex free loud hill next sets",
            "part jolt saga twin quiz work taxi puma idea what belt data mint bald road twin fund hope axis saga",
            "taxi judo even jugs cost hawk redo ruby menu exit fair ruin very cola fish memo tiny jade quiz hang",
            "chef swan back also visa hawk peck kite keep dull silk gala heat bulb quiz holy jade drum memo vibe",
            "gyro lion gift hawk gift rich jury lung belt epic away nail body math zinc luck part nail jury inky",
            "skew mild trip hang able able work jugs miss blue miss cola wolf beta onyx king numb buzz axis curl",
            "time gray yell user down data deli keys trip each fact jade huts keep runs love quad chef purr memo",
        ];

        let expected_id_words = [
            "INCH ONYX USER RUNS",
            "GLOW BRAG BUZZ TWIN",
            "IRON ZINC GIRL DOWN",
            "LUAU EPIC LION DARK",
            "FERN KENO KNOB EVEN",
            "GOOD APEX RUIN MEMO",
            "CATS HELP JOIN SLOT",
            "ITEM WOLF DOOR FOXY",
            "FAIR JOWL GAME DATA",
            "SAFE LEGS RUNS HOPE",
        ];

        let expected_bytemoji_ids = [
            "🪴 📦 🐼 🎺",
            "🫒 😳 😡 🐭",
            "🍁 🐟 🥕 😿",
            "🔷 👄 🚩 👻",
            "🦶 🌞 🪐 🦷",
            "🧄 😉 🎷 🚜",
            "🤠 🌮 🌼 👕",
            "🍄 🐺 🙀 🫐",
            "🤚 🌸 🥑 👽",
            "🏈 💖 🎺 🍤",
        ];

        let expected_urs = [
            "ur:provenance/lfaegdtokebznlahftbsnlaxpsdiwecswsrnlsdsdpghrp",
            "ur:provenance/lfaegdinhlpeoyfsfnbgpdsklfbetkmetbhgcthpmeeoos",
            "ur:provenance/lfaegdettkvwetstjprokpbdkkiykkcedrtkcedstblpds",
            "ur:provenance/lfaegdrnuyoespsronaaosaoihurmwplleaxfecwhhfstb",
            "ur:provenance/lfaegdptjtsatnqzwktipaiawtbtdamtbdrdtntnhyptti",
            "ur:provenance/lfaegdtijoenjscthkrorymuetfrrnvycafhmofgimbbfe",
            "ur:provenance/lfaegdcfsnbkaovahkpkkekpdlskgahtbbqzhyytdleyyn",
            "ur:provenance/lfaegdgolngthkgtrhjylgbtecaynlbymhzclkfrmktyjy",
            "ur:provenance/lfaegdswmdtphgaeaewkjsmsbemscawfbaoxkgeybbpteo",
            "ur:provenance/lfaegdtegyylurdndadikstpehftjehskprsleclcsbgla",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfaegdtokebznlahftbsnlaxpsdiwecswsrnlsvtierfwl",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdinhlpeoyfsfnbgpdsklfbetkmetbhgctnttpuyya",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdettkvwetstjprokpbdkkiykkcedrtkcevtnejnkk",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdrnuyoespsronaaosaoihurmwplleaxfeutbztlld",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdptjtsatnqzwktipaiawtbtdamtbdrdtncechfpmy",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdtijoenjscthkrorymuetfrrnvycafhmolacnztcy",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdcfsnbkaovahkpkkekpdlskgahtbbqzhyfhiytnpt",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdgolngthkgtrhjylgbtecaynlbymhzclkzcttfndn",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdswmdtphgaeaewkjsmsbemscawfbaoxkgwkhlfpjz",
            "https://example.com/validate?provenance=tngdgmgwhflfaegdtegyylurdndadikstpehftjehskprslevdgyzsur",
        ];

        run_test(
            ProvenanceMarkResolution::Low,
            false,
            &expected_descriptions,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
            false
        );
    }

    #[test]
    fn test_low_with_info() {
        let expected_descriptions = [
            r#"ProvenanceMark(key: ce7c1599, hash: c9929b6e, chainID: ce7c1599, seq: 0, date: 2023-06-20T00:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 695dafa1, hash: dec86566, chainID: ce7c1599, seq: 1, date: 2023-06-21T00:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 38cfe538, hash: 08677f72, chainID: ce7c1599, seq: 2, date: 2023-06-22T00:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: bedba2c8, hash: e736559f, chainID: ce7c1599, seq: 3, date: 2023-06-23T00:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: a96ec2da, hash: 9b8a5879, chainID: ce7c1599, seq: 4, date: 2023-06-24T00:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: d0703671, hash: 2c68ee9b, chainID: ce7c1599, seq: 5, date: 2023-06-25T00:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 19cd0a02, hash: 48955250, chainID: ce7c1599, seq: 6, date: 2023-06-26T00:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 55864d59, hash: eb2f7fc9, chainID: ce7c1599, seq: 7, date: 2023-06-27T00:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: c695d857, hash: 3e142656, chainID: ce7c1599, seq: 8, date: 2023-06-28T00:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: d351f7df, hash: 4831416c, chainID: ce7c1599, seq: 9, date: 2023-06-29T00:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
        ];

        let expected_bytewords = [
            "taco kite buzz nail arch fact bias nail pose navy idea fern cats webs ruin legs swan half hill guru belt omit fish menu quad jowl zinc logo hang lava vast lazy puff puma cash visa wand jury news ramp swan wolf cola user data kite time fact tied",
            "iron hill pose obey figs fern brag paid gush hard horn junk maze tomb hang cost item poem user good yell gift high list chef slot very song rock junk puma taxi curl flap mint rich soap wave bulb yoga fair many lung zone oboe ugly lion peck gray",
            "exit task view exit slot jump redo keep item vial hang crux code door task code holy tomb barn hawk veto work holy away onyx flew dice vows webs item zoom what what judo yoga meow cost yell trip hill wasp tiny eyes join pose hang note inch cyan",
            "ruin ugly oboe soap scar open aqua owls jolt inky barn dull pool love apex free wolf trip foxy iron yoga jolt puff play code puma judo jolt keep owls visa bias pool yawn dark what drop atom tuna diet flew lion gala when legs arch purr diet huts",
            "part jolt saga twin quiz work taxi puma sets barn down item mint bald road twin barn cook away heat flux half hope trip twin note meow time news navy wave meow dark taxi rock tiny bulb fuel what work rock gems part saga zinc safe news next zinc",
            "taxi judo even jugs cost hawk redo ruby webs guru jade real very cola fish memo wasp horn iris loud code real idle flap onyx tent each junk memo curl silk rust runs many gala atom gear wasp cola lamb toys even yank gala luck epic miss toil taco",
            "chef swan back also visa hawk peck kite data very zaps urge heat bulb quiz holy user runs wasp toil skew axis calm webs belt cusp play lamb acid taxi vows exam also blue memo silk inch skew quad peck solo aqua figs even yell redo days tent noon",
            "gyro lion gift hawk gift rich jury lung luck wall hill bulb body math zinc luck whiz solo echo main frog vibe wall into taxi epic lamb plus luck arch claw soap hill able real liar fuel quad runs rust miss mild zone bald news ruby glow flew exit",
            "skew mild trip hang able able work jugs memo jade zero jolt wolf beta onyx king need play body away atom fact arch join many echo body each epic rich omit high puff fund eyes urge silk aqua help view rust iron crux urge fact fuel hard edge king",
            "time gray yell user down data deli keys gray legs sets hard huts keep runs love cusp surf pose gems numb user vows loud liar taxi zaps eyes oval paid real work jolt keno bias paid puff wolf gems down peck kept note part lion tuna oboe webs join",
        ];

        let expected_id_words = [
            "SOLO MEMO NEED JOLT",
            "URGE SOAP INCH INKY",
            "AWAY INTO LAMB JUMP",
            "VOID EVEN GYRO NOTE",
            "NEED LOVE HARD KICK",
            "DRAW IRIS WAXY NEED",
            "FUND MILD GRIM GOOD",
            "WARM DULL LAMB SOLO",
            "FILM BULB DAYS HALF",
            "FUND EACH FLAP JAZZ",
        ];

        let expected_bytemoji_ids = [
            "👖 🚜 🎢 🌻",
            "🐻 👚 🪴 🌵",
            "😘 🌱 🌐 💧",
            "🐔 🦷 🥚 🔑",
            "🎢 🔴 🍔 🌜",
            "🫶 💐 🐛 🎢",
            "🥝 🚀 🥯 🧄",
            "🐴 🤝 🌐 👖",
            "🍊 😵 😺 🍗",
            "🥝 👎 🍉 🌺",
        ];

        let expected_urs = [
            "ur:provenance/lfaehddptokebznlahftbsnlpenyiafncswsrnlssnhfhlgubtotfhmuqdjlzclohglavtlypfpachvawdjynsrpsnwfcaurdalyrffnsf",
            "ur:provenance/lfaehddpinhlpeoyfsfnbgpdghhdhnjkmetbhgctimpmurgdylgthhltcfstvysgrkjkpaticlfpmtrhspwebbyafrmylgzeoedswlpsgw",
            "ur:provenance/lfaehddpettkvwetstjprokpimvlhgcxcedrtkcehytbbnhkvowkhyayoxfwdevswsimzmwtwtjoyamwctyltphlwptyesjnpepkwtiafs",
            "ur:provenance/lfaehddprnuyoespsronaaosjtiybndlplleaxfewftpfyinyajtpfpycepajojtkposvabsplyndkwtdpamtadtfwlngawnlsyautdllb",
            "ur:provenance/lfaehddpptjtsatnqzwktipassbndnimmtbdrdtnbnckayhtfxhfhetptnnemwtensnywemwdktirktybbflwtwkrkgsptsazcfnwfndvl",
            "ur:provenance/lfaehddptijoenjscthkrorywsgujerlvycafhmowphnisldcerliefpoxttehjkmoclskrtrsmygaamgrwpcalbtsenykgalkspyateti",
            "ur:provenance/lfaehddpcfsnbkaovahkpkkedavyzsuehtbbqzhyurrswptlswascmwsbtcppylbadtivsemaobemoskihswqdpksoaafsenylfegatsla",
            "ur:provenance/lfaehddpgolngthkgtrhjylglkwlhlbbbymhzclkwzsoeomnfgvewliotieclbpslkahcwsphlaerllrflqdrsrtmsmdzebdnsfzcxfyds",
            "ur:provenance/lfaehddpswmdtphgaeaewkjsmojezojtwfbaoxkgndpybyayamftahjnmyeobyehecrhothhpffdesueskaahpvwrtincxueftrdemeyih",
            "ur:provenance/lfaehddptegyylurdndadiksgylssshdhskprslecpsfpegsnburvsldlrtizsesolpdrlwkjtkobspdpfwfgsdnpkktneptlndksnwljk",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfaehddptokebznlahftbsnlpenyiafncswsrnlssnhfhlgubtotfhmuqdjlzclohglavtlypfpachvawdjynsrpsnwfcaurdaiylsdlcp",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpinhlpeoyfsfnbgpdghhdhnjkmetbhgctimpmurgdylgthhltcfstvysgrkjkpaticlfpmtrhspwebbyafrmylgzeoesetbrsoy",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpettkvwetstjprokpimvlhgcxcedrtkcehytbbnhkvowkhyayoxfwdevswsimzmwtwtjoyamwctyltphlwptyesjnpegttkjote",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddprnuyoespsronaaosjtiybndlplleaxfewftpfyinyajtpfpycepajojtkposvabsplyndkwtdpamtadtfwlngawnlsctvofnme",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpptjtsatnqzwktipassbndnimmtbdrdtnbnckayhtfxhfhetptnnemwtensnywemwdktirktybbflwtwkrkgsptsazcuysflobt",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddptijoenjscthkrorywsgujerlvycafhmowphnisldcerliefpoxttehjkmoclskrtrsmygaamgrwpcalbtsenykgalkdlstrtfm",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpcfsnbkaovahkpkkedavyzsuehtbbqzhyurrswptlswascmwsbtcppylbadtivsemaobemoskihswqdpksoaafsenyloekossjt",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpgolngthkgtrhjylglkwlhlbbbymhzclkwzsoeomnfgvewliotieclbpslkahcwsphlaerllrflqdrsrtmsmdzebdnsoscthgsp",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddpswmdtphgaeaewkjsmojezojtwfbaoxkgndpybyayamftahjnmyeobyehecrhothhpffdesueskaahpvwrtincxuefthlaycllu",
            "https://example.com/validate?provenance=tngdgmgwhflfaehddptegyylurdndadiksgylssshdhskprslecpsfpegsnburvsldlrtizsesolpdrlwkjtkobspdpfwfgsdnpkktneptlnsrwzzsnt",
        ];

        run_test(
            ProvenanceMarkResolution::Low,
            true,
            &expected_descriptions,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
            false
        );
    }

    #[test]
    fn test_medium() {
        let expected_descriptions = [
            r#"ProvenanceMark(key: ce7c1599b0506f5f, hash: 5a225674d1827609, chainID: ce7c1599b0506f5f, seq: 0, date: 2023-06-20T12:00:00Z)"#,
            r#"ProvenanceMark(key: 695dafa138cfe538, hash: cdf2d4eb79b4a4da, chainID: ce7c1599b0506f5f, seq: 1, date: 2023-06-21T12:00:00Z)"#,
            r#"ProvenanceMark(key: bedba2c8a96ec2da, hash: 9fd023a34ea727b1, chainID: ce7c1599b0506f5f, seq: 2, date: 2023-06-22T12:00:00Z)"#,
            r#"ProvenanceMark(key: d070367119cd0a02, hash: d2acd98fcc5f909e, chainID: ce7c1599b0506f5f, seq: 3, date: 2023-06-23T12:00:00Z)"#,
            r#"ProvenanceMark(key: 55864d59c695d857, hash: a9ad894b6ed463d1, chainID: ce7c1599b0506f5f, seq: 4, date: 2023-06-24T12:00:00Z)"#,
            r#"ProvenanceMark(key: d351f7dff419008f, hash: 08f81843d58cff43, chainID: ce7c1599b0506f5f, seq: 5, date: 2023-06-25T12:00:00Z)"#,
            r#"ProvenanceMark(key: 691d0bebe4e71f69, hash: 905333b27d92e07a, chainID: ce7c1599b0506f5f, seq: 6, date: 2023-06-26T12:00:00Z)"#,
            r#"ProvenanceMark(key: bfd291fd7e6eb4df, hash: 091eeeafd036128f, chainID: ce7c1599b0506f5f, seq: 7, date: 2023-06-27T12:00:00Z)"#,
            r#"ProvenanceMark(key: f86f78ab260ce12c, hash: 808d25bfd9fdc615, chainID: ce7c1599b0506f5f, seq: 8, date: 2023-06-28T12:00:00Z)"#,
            r#"ProvenanceMark(key: 650a700450011d2f, hash: 14e5ac077acffa1f, chainID: ce7c1599b0506f5f, seq: 9, date: 2023-06-29T12:00:00Z)"#,
        ];

        let expected_bytewords = [
            "taco kite buzz nail puff good jowl hope kite even sets zero bald inky lazy frog jury claw buzz bald item axis memo arch wave judo veto jolt jugs hope each glow zone urge eyes wasp",
            "iron hill pose obey exit task view exit cats blue beta film very data purr meow scar inky girl cats yoga song road beta draw next song math kite unit solo gear redo cash chef zinc",
            "ruin ugly oboe soap part jolt saga twin oboe loud junk beta what need webs kick love saga whiz inch twin lazy holy tiny what zaps twin apex acid miss days tiny kept ruin many scar",
            "taxi judo even jugs chef swan back also fund high horn belt fern brew lung ruin iris huts cats zoom knob jugs inch toil calm race monk gala gift days gray taxi gush whiz keep safe",
            "gyro lion gift hawk skew mild trip hang leaf taco zaps iron yank dull even main body zero days warm down rich gray vial safe kiwi news taxi news able hill yank lion lazy pool leaf",
            "time gray yell user work chef able many acid gyro also swan rich aqua poem soap mint king dull zoom part also judo play deli huts task bulb lazy maze fund epic tuna work ramp back",
            "iron cola bald warm vibe void cost iron road memo plus puma blue paid code girl tent cash toys urge oval gems bald heat list puff view oval solo zaps body wand wall good paid hang",
            "runs tied maze zinc knob jolt quiz user heat roof fish road rust waxy yoga acid kiln onyx quiz curl yank menu owls maze waxy oboe unit soap curl atom wasp waxy keno redo veto fish",
            "yoga jowl keys play days barn very draw tomb roof undo cash zone aunt easy barn logo gala gray item nail acid curl claw next curl need jade love knob junk scar wolf idle edge quad",
            "inch back judo aqua good acid cola dull rock hill monk ugly iris buzz down wand aunt race meow zaps epic film lamb gray edge film flux race frog jade holy warm taco calm flap echo",
        ];

        let expected_id_words = [
            "HEAT CUSP HALF JURY",
            "SWAN WHIZ TINY WARM",
            "NOTE TAXI CYAN OMIT",
            "TIED PLUS TUNA MANY",
            "PART POEM LOUD GEAR",
            "AWAY YOGA CATS FLUX",
            "MATH GURU ECHO PURR",
            "AXIS COOK WAXY POSE",
            "LAVA LUNG DATA RUNS",
            "BULB VIEW PLUS AUNT",
        ];

        let expected_bytemoji_ids = [
            "🍕 👺 🍗 🌀",
            "🧢 🐢 🧦 🐴",
            "🔑 🧵 💀 💌",
            "👠 📷 🐶 🚗",
            "🧮 ⏰ 🚫 🥦",
            "😘 🪼 🤠 🍓",
            "🚑 🍞 👆 🧲",
            "😭 🙃 🐛 📡",
            "💛 🛑 👽 🎺",
            "😵 🐸 📷 😍",
        ];

        let expected_urs = [
            "ur:provenance/lfadhdcxtokebznlpfgdjlhekeensszobdiylyfgjycwbzbdimasmoahwejovojtjsheehgwwtgansde",
            "ur:provenance/lfadhdcxinhlpeoyettkvwetcsbebafmvydaprmwsriyglcsyasgrdbadwntsgmhkeutsogrrplarfes",
            "ur:provenance/lfadhdcxrnuyoespptjtsatnoeldjkbawtndwskklesawzihtnlyhytywtzstnaxadmsdstykkdtdrat",
            "ur:provenance/lfadhdcxtijoenjscfsnbkaofdhhhnbtfnbwlgrnishscszmkbjsihtlcmremkgagtdsgytihtihtiah",
            "ur:provenance/lfadhdcxgolngthkswmdtphglftozsinykdlenmnbyzodswmdnrhgyvlsekinstinsaehlyklocmbdfg",
            "ur:provenance/lfadhdcxtegyylurwkcfaemyadgoaosnrhaapmspmtkgdlzmptaojopydihstkbblymefdectsiabwto",
            "ur:provenance/lfadhdcxincabdwmvevdctinrdmopspabepdceglttchtsueolgsbdhtltpfvwolsozsbywdvdstbtmu",
            "ur:provenance/lfadhdcxrstdmezckbjtqzurhtrffhrdrtwyyaadknoxqzclykmuosmewyoeutspclamwpwyksdlflzo",
            "ur:provenance/lfadhdcxyajlkspydsbnvydwtbrfuochzeateybnlogagyimnladclcwntclndjelekbjksrzcwfmekt",
            "ur:provenance/lfadhdcxihbkjoaagdadcadlrkhlmkuyisbzdnwdatremwzsecfmlbgyeefmfxrefgjehywmrtlyveyl",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxtokebznlpfgdjlhekeensszobdiylyfgjycwbzbdimasmoahwejovojtjsheehgwwpwzmuzm",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxinhlpeoyettkvwetcsbebafmvydaprmwsriyglcsyasgrdbadwntsgmhkeutsogrpkfrqdwy",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxrnuyoespptjtsatnoeldjkbawtndwskklesawzihtnlyhytywtzstnaxadmsdstyihmodati",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxtijoenjscfsnbkaofdhhhnbtfnbwlgrnishscszmkbjsihtlcmremkgagtdsgytifgueurtd",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxgolngthkswmdtphglftozsinykdlenmnbyzodswmdnrhgyvlsekinstinsaehlykmwpmaame",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxtegyylurwkcfaemyadgoaosnrhaapmspmtkgdlzmptaojopydihstkbblymefdecsbtpcecf",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxincabdwmvevdctinrdmopspabepdceglttchtsueolgsbdhtltpfvwolsozsbywdzokeaofy",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxrstdmezckbjtqzurhtrffhrdrtwyyaadknoxqzclykmuosmewyoeutspclamwpwyiemwfddw",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxyajlkspydsbnvydwtbrfuochzeateybnlogagyimnladclcwntclndjelekbjksrvyfdnnnb",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdcxihbkjoaagdadcadlrkhlmkuyisbzdnwdatremwzsecfmlbgyeefmfxrefgjehywmuoftwmcx",
        ];

        run_test(
            ProvenanceMarkResolution::Medium,
            false,
            &expected_descriptions,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
            false
        );
    }

    #[test]
    fn test_medium_with_info() {
        let expected_descriptions = [
            r#"ProvenanceMark(key: ce7c1599b0506f5f, hash: 447f1063fccdc8f4, chainID: ce7c1599b0506f5f, seq: 0, date: 2023-06-20T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 695dafa138cfe538, hash: b18f5360ead0b3cc, chainID: ce7c1599b0506f5f, seq: 1, date: 2023-06-21T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: bedba2c8a96ec2da, hash: 523f9657cbd05eff, chainID: ce7c1599b0506f5f, seq: 2, date: 2023-06-22T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: d070367119cd0a02, hash: 740467a1bb169873, chainID: ce7c1599b0506f5f, seq: 3, date: 2023-06-23T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 55864d59c695d857, hash: 5fab738f6fb484ca, chainID: ce7c1599b0506f5f, seq: 4, date: 2023-06-24T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: d351f7dff419008f, hash: 199bc28eda93972c, chainID: ce7c1599b0506f5f, seq: 5, date: 2023-06-25T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 691d0bebe4e71f69, hash: fa427253e2bc6be5, chainID: ce7c1599b0506f5f, seq: 6, date: 2023-06-26T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: bfd291fd7e6eb4df, hash: 2d0cb2947bb0a1e3, chainID: ce7c1599b0506f5f, seq: 7, date: 2023-06-27T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: f86f78ab260ce12c, hash: 7b684ef10022b749, chainID: ce7c1599b0506f5f, seq: 8, date: 2023-06-28T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 650a700450011d2f, hash: d7c671e2aadcead0, chainID: ce7c1599b0506f5f, seq: 9, date: 2023-06-29T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
        ];

        let expected_bytewords = [
            "taco kite buzz nail puff good jowl hope kite even sets zero bald inky lazy frog item frog guru code fuel frog draw yoga wave judo veto jolt jugs hope each glow very item monk dice film obey rock zero join idle jowl runs quad axis horn dark flux tied gear junk good city buzz axis item meow gala next numb luau vial data when",
            "iron hill pose obey exit task view exit cats blue beta film very data purr meow runs claw solo menu jade pool poem cats draw next song math kite unit solo gear cash able real fizz drop zinc cusp into fizz numb meow figs stub monk guru jury quad mint jury yawn meow iced list cusp saga kiln yoga obey cash iced visa jugs kept",
            "ruin ugly oboe soap part jolt saga twin oboe loud junk beta what need webs kick fuel drop fuel maze hope yawn deli navy what zaps twin apex acid miss days tiny luau item horn heat hill hope numb plus lion down luau logo gift solo glow mild guru stub list mild ruby echo into keno puff lava peck ruby kiwi yell door atom what",
            "taxi judo even jugs chef swan back also fund high horn belt fern brew lung ruin taco solo oval tent axis exit join exit calm race monk gala gift days gray taxi jury zinc gala gift gala dark able bias what cost many edge real gray undo lava purr gems down very hard guru buzz buzz owls road leaf rich pose wave sets vibe legs",
            "gyro lion gift hawk skew mild trip hang leaf taco zaps iron yank dull even main void zinc undo dull door tuna ramp yoga safe kiwi news taxi news able hill yank back edge tuna game when gems rock wave mint monk jade bulb yell free waxy bias jury yurt frog unit keys dice buzz even limp bias navy puff aqua race soap item back",
            "time gray yell user work chef able many acid gyro also swan rich aqua poem soap list cats yank easy oval cola cats sets deli huts task bulb lazy maze fund epic owls dull drop fuel cola kick free poem cats loud many barn city draw wall real gyro zinc flux lazy cyan fair jugs down drum flap deli news ruin idle runs next epic",
            "iron cola bald warm vibe void cost iron road memo plus puma blue paid code girl rock atom mint fish eyes iced lava silk list puff view oval solo zaps body wand swan redo silk skew wolf acid dice twin liar lung pool iron plus view note drum cash gyro gems love pool zest drop half drop paid iris bulb wasp good toil twin miss",
            "runs tied maze zinc knob jolt quiz user heat roof fish road rust waxy yoga acid holy ramp vows city holy buzz bulb zinc waxy oboe unit soap curl atom wasp waxy vibe twin liar next dull hang trip rock flew drum math zinc ruin junk jolt item guru urge aunt need zero navy twin oval apex safe void frog leaf omit void frog twin",
            "yoga jowl keys play days barn very draw tomb roof undo cash zone aunt easy barn junk plus fact dark fizz urge good fuel next curl need jade love knob junk scar part lamb figs need item toil epic slot arch roof limp yurt half dull toil work peck rock maze claw huts dark ramp horn edge chef each math iron yank able scar keno",
            "inch back judo aqua good acid cola dull rock hill monk ugly iris buzz down wand sets mint gala cost view drop jowl noon edge film flux race frog jade holy warm paid draw veto vial hawk math glow tent owls kiwi hard away skew brew also acid zoom idea monk iced jowl note horn drum puff saga scar epic kite time flap king time",
        ];

        let expected_id_words = [
            "FOXY LAMB BLUE IDEA",
            "PUMA MANY GURU HORN",
            "GRIM FISH MINT HANG",
            "JURY AQUA INTO OBEY",
            "HOPE PLAY JUNK MANY",
            "CHEF NEED SAGA MAIN",
            "ZAPS FLEW JUMP GURU",
            "DROP BARN PURR MEOW",
            "KING IRIS GIRL WHEN",
            "TOYS SKEW JUGS VETO",
        ];

        let expected_bytemoji_ids = [
            "🫐 🌐 🥵 🍦",
            "💰 🚗 🍞 🍚",
            "🥯 🍋 🚁 🌭",
            "🌀 🙄 🌱 🪑",
            "🍤 💎 💦 🚗",
            "🤡 🎢 🎾 🔺",
            "🦀 🍇 💧 🍞",
            "🤲 🤩 🧲 🚨",
            "🌎 💐 🥕 🐞",
            "👜 💥 🌊 🦁",
        ];

        let expected_urs = [
            "ur:provenance/lfadhdfstokebznlpfgdjlhekeensszobdiylyfgimfgguceflfgdwyawejovojtjsheehgwvyimmkdefmoyrkzojniejlrsqdashndkfxtdgrjkgdcybzasimmwgantnbutbwdyhy",
            "ur:provenance/lfadhdfsinhlpeoyettkvwetcsbebafmvydaprmwrscwsomujeplpmcsdwntsgmhkeutsogrchaerlfzdpzccpiofznbmwfssbmkgujyqdmtjyynmwidltcpsaknyaoycheecmietp",
            "ur:provenance/lfadhdfsrnuyoespptjtsatnoeldjkbawtndwskkfldpflmeheyndinywtzstnaxadmsdstyluimhnhthlhenbpslndnlulogtsogwmdgusbltmdryeoiokopflapkrykioytnbwhe",
            "ur:provenance/lfadhdfstijoenjscfsnbkaofdhhhnbtfnbwlgrntosoolttasetjnetcmremkgagtdsgytijyzcgagtgadkaebswtctmyeerlgyuolaprgsdnvyhdgubzbzosrdlfrhperkeewndw",
            "ur:provenance/lfadhdfsgolngthkswmdtphglftozsinykdlenmnvdzcuodldrtarpyasekinstinsaehlykbkeetagewngsrkwemtmkjebbylfewybsjyytfgutksdebzenlpbsnypfaavletlbon",
            "ur:provenance/lfadhdfstegyylurwkcfaemyadgoaosnrhaapmspltcsykeyolcacsssdihstkbblymefdecosdldpflcakkfepmcsldmybncydwwlrlgozcfxlycnfrjsdndmfpdinsrneygwlony",
            "ur:provenance/lfadhdfsincabdwmvevdctinrdmopspabepdceglrkammtfhesidlaskltpfvwolsozsbywdsnroskswwfaddetnlrlgplinpsvwnedmchgogsleplztdphfdppdisbbwpamdatket",
            "ur:provenance/lfadhdfsrstdmezckbjtqzurhtrffhrdrtwyyaadhyrpvscyhybzbbzcwyoeutspclamwpwyvetnlrntdlhgtprkfwdmmhzcrnjkjtimguueatndzonytnolaxsevdfglfykchgukp",
            "ur:provenance/lfadhdfsyajlkspydsbnvydwtbrfuochzeateybnjkpsftdkfzuegdflntclndjelekbjksrptlbfsndimtlecstahrflpythfdltlwkpkrkmecwhsdkrphneecfehmhinotwttbta",
            "ur:provenance/lfadhdfsihbkjoaagdadcadlrkhlmkuyisbzdnwdssmtgactvwdpjlnneefmfxrefgjehywmpddwvovlhkmhgwttoskihdayswbwaoadzmiamkidjlnehndmpfsasreckelppajtke",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfstokebznlpfgdjlhekeensszobdiylyfgimfgguceflfgdwyawejovojtjsheehgwvyimmkdefmoyrkzojniejlrsqdashndkfxtdgrjkgdcybzasimmwgantnbvlnyhtvl",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsinhlpeoyettkvwetcsbebafmvydaprmwrscwsomujeplpmcsdwntsgmhkeutsogrchaerlfzdpzccpiofznbmwfssbmkgujyqdmtjyynmwidltcpsaknyaoychbknebaih",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsrnuyoespptjtsatnoeldjkbawtndwskkfldpflmeheyndinywtzstnaxadmsdstyluimhnhthlhenbpslndnlulogtsogwmdgusbltmdryeoiokopflapkrykinegukkvo",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfstijoenjscfsnbkaofdhhhnbtfnbwlgrntosoolttasetjnetcmremkgagtdsgytijyzcgagtgadkaebswtctmyeerlgyuolaprgsdnvyhdgubzbzosrdlfrhpelpryndme",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsgolngthkswmdtphglftozsinykdlenmnvdzcuodldrtarpyasekinstinsaehlykbkeetagewngsrkwemtmkjebbylfewybsjyytfgutksdebzenlpbsnypfaautpabzcs",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfstegyylurwkcfaemyadgoaosnrhaapmspltcsykeyolcacsssdihstkbblymefdecosdldpflcakkfepmcsldmybncydwwlrlgozcfxlycnfrjsdndmfpdinsrnbnswvodi",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsincabdwmvevdctinrdmopspabepdceglrkammtfhesidlaskltpfvwolsozsbywdsnroskswwfaddetnlrlgplinpsvwnedmchgogsleplztdphfdppdisbbwpetpsonlp",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsrstdmezckbjtqzurhtrffhrdrtwyyaadhyrpvscyhybzbbzcwyoeutspclamwpwyvetnlrntdlhgtprkfwdmmhzcrnjkjtimguueatndzonytnolaxsevdfglfsbnnessp",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsyajlkspydsbnvydwtbrfuochzeateybnjkpsftdkfzuegdflntclndjelekbjksrptlbfsndimtlecstahrflpythfdltlwkpkrkmecwhsdkrphneecfehmhinntkkrfie",
            "https://example.com/validate?provenance=tngdgmgwhflfadhdfsihbkjoaagdadcadlrkhlmkuyisbzdnwdssmtgactvwdpjlnneefmfxrefgjehywmpddwvovlhkmhgwttoskihdayswbwaoadzmiamkidjlnehndmpfsasreckerketaase",
        ];

        run_test(
            ProvenanceMarkResolution::Medium,
            true,
            &expected_descriptions,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
            false
        );
    }

    #[test]
    fn test_quartile() {
        let expected_descriptions = [
            r#"ProvenanceMark(key: ce7c1599b0506f5f9091e0fca796a4f3, hash: 519f20b6b72384595fe3d6258a09511f, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 0, date: 2023-06-20T12:00:00Z)"#,
            r#"ProvenanceMark(key: 695dafa138cfe538bedba2c8a96ec2da, hash: 6152368e7f49e52f80c9e21f4defac1d, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 1, date: 2023-06-21T12:00:00Z)"#,
            r#"ProvenanceMark(key: d070367119cd0a0255864d59c695d857, hash: edf4286385fc90e8e3972cdf37cfd77f, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 2, date: 2023-06-22T12:00:00Z)"#,
            r#"ProvenanceMark(key: d351f7dff419008f691d0bebe4e71f69, hash: b40ff1c26da0b6f8efaa96226010c9d1, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 3, date: 2023-06-23T12:00:00Z)"#,
            r#"ProvenanceMark(key: bfd291fd7e6eb4dff86f78ab260ce12c, hash: 66c5c63c4d5122117d87d50977767203, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 4, date: 2023-06-24T12:00:00Z)"#,
            r#"ProvenanceMark(key: 650a700450011d2fea8a9bc2249af6c2, hash: 93fa96c9ea09456bf6ccdd4144e1e2ae, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 5, date: 2023-06-25T12:00:00Z)"#,
            r#"ProvenanceMark(key: 24539e315edbdc34b0dd5361956328ca, hash: a2e706926d03b9876b87568350889204, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 6, date: 2023-06-26T12:00:00Z)"#,
            r#"ProvenanceMark(key: 869c390f34b1f7e0d618ba6b3f999a0e, hash: 225286e2ccbc61e7cc33549478402e9c, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 7, date: 2023-06-27T12:00:00Z)"#,
            r#"ProvenanceMark(key: e1929c31e0f8c8c5e0b74cbbb4fdba35, hash: ddc425d94942c16a5d132b336c82b9df, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 8, date: 2023-06-28T12:00:00Z)"#,
            r#"ProvenanceMark(key: 9d9959631c2d8991161a8a5bec17edb2, hash: 4f6161ce9e3922ccb2bd3188fbce816e, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 9, date: 2023-06-29T12:00:00Z)"#,
        ];

        let expected_bytewords = [
            "taco kite buzz nail puff good jowl hope math maze vast zest owls mint onyx wolf bulb many mint atom list gift mild tuna hang hope yurt silk poem jump chef flap toil drop epic iced gift vows quad holy cyan owls axis hill data cola away twin code dice exit iced apex gush frog fact join iris open exit aunt jazz",
            "iron hill pose obey exit task view exit ruin ugly oboe soap part jolt saga twin horn duty loud lamb jowl cyan view huts game need jade frog vows tiny edge roof judo flew atom curl time arch view view even gyro onyx time jugs zoom skew urge crux lamb huts liar zoom skew heat kite runs what yank need jade zinc",
            "taxi judo even jugs chef swan back also gyro lion gift hawk skew mild trip hang kiwi task part loud work fuel join dice unit vibe race vibe claw cusp lung girl what vibe dull menu away echo zest junk luau runs road purr next figs very help jolt claw drop foxy lazy obey very help item gala knob vibe barn glow",
            "time gray yell user work chef able many iron cola bald warm vibe void cost iron guru open fact yoga cyan fund dark cook runs frog whiz lamb stub glow task saga easy owls draw sets trip game liar noon plus able keep paid keno mild game horn judo door yoga poem navy calm lung nail drum apex twin scar glow wasp",
            "runs tied maze zinc knob jolt quiz user yoga jowl keys play days barn very draw echo jugs navy paid hard join liar bias yank zone fern maze purr atom road fern horn puma junk eyes also gear main whiz silk memo jugs lion zinc gear puff idle skew idea inch gray main into heat fact keno drum wasp claw cyan oboe",
            "inch back judo aqua good acid cola dull wand love need saga dark navy yawn saga beta acid gear crux tied exam tuna fair keys oboe diet tomb lion wave kiwi lava silk tied fund paid onyx able epic scar cook hang skew list view jade liar wave time lazy toil nail logo rock kiwi toys news yoga dark idle ruby ruin",
            "dark guru noon each holy ugly undo edge puff unit guru huts mild idea dice song epic edge urge iris atom ramp film vast data oboe cost undo list arch yell dice drum twin purr wave void trip guru owls plus dice quad lava peck gyro foxy city drop loud jazz onyx jugs inky slot join twin inch jugs peck yurt foxy",
            "lion news eyes bias edge puma yell vast tomb cats road jade fish nail navy beta help very jade wolf wall brew curl guru task gift navy zone fern part door acid stub figs free undo holy idea pool obey ugly barn fair brag kick iris quiz visa toys iris buzz idea webs beta zoom open note mint hawk drum vast fish",
            "very memo news each vast yoga soap silk vast real gems rock quiz zinc road epic iris vows gear cash math vows heat many real navy bias axis zinc cost puma kick love eyes miss vial meow mild wolf exam tuna huts idea view mild puff cash gyro taxi twin what redo days hill jugs claw beta idle code user ruin cash",
            "next nail hawk idea code drop loud maze calm city love help wasp cash wave purr navy bald fair atom surf fizz aunt zoom exit draw navy flux skew cusp mint task user zaps aunt fair jump bias hard part inky claw puff nail eyes iris obey cost bias rich miss idle jury vibe oboe wand good yell drum away mint each",
        ];

        let expected_id_words = [
            "GRAY NOTE CRUX RAMP",
            "HUTS GRIM EVEN MAIN",
            "WAVE WORK DICE IDEA",
            "QUIZ BIAS WHEN SAGA",
            "INKY SILK SKEW FERN",
            "MENU ZAPS MINT SOLO",
            "OBOE VOID ATOM MEMO",
            "CUSP GRIM LION VETO",
            "UNIT SETS DATA TUNA",
            "GLOW HUTS HUTS TACO",
        ];

        let expected_bytemoji_ids = [
            "🥐 🔑 😈 🎉",
            "🥠 🥯 🦷 🔺",
            "🐝 🐍 😻 🍦",
            "🎁 🫥 🐞 🎾",
            "🌵 🔥 💥 🦶",
            "🛵 🦀 🚁 👖",
            "🎈 🐔 😎 🚜",
            "👺 🥯 🚩 🦁",
            "🦊 ✨ 👽 🐶",
            "🫒 🥠 🥠 👓",
        ];

        let expected_urs = [
            "ur:provenance/lfaohdfttokebznlpfgdjlhemhmevtztosmtoxwfbbmymtamltgtmdtahgheytskpmjpcffptldpecidgtvsqdhycnosashldacaaytncedeetidaxghfgftjnisfmaepdlk",
            "ur:provenance/lfaohdftinhlpeoyettkvwetrnuyoespptjtsatnhndyldlbjlcnvwhsgendjefgvstyeerfjofwamclteahvwvwengooxtejszmswuecxlbhslrzmswhtkerswtjtotssca",
            "ur:provenance/lfaohdfttijoenjscfsnbkaogolngthkswmdtphgkitkptldwkfljndeutverevecwcplgglwtvedlmuayeoztjklursrdprntfsvyhpjtcwdpfylyoyvyhpimgavwuootpe",
            "ur:provenance/lfaohdfttegyylurwkcfaemyincabdwmvevdctinguonftyacnfddkckrsfgwzlbsbgwtksaeyosdwsstpgelrnnpsaekppdkomdgehnjodryapmnycmlgnldmaxfpzovtbn",
            "ur:provenance/lfaohdftrstdmezckbjtqzuryajlkspydsbnvydweojsnypdhdjnlrbsykzefnmepramrdfnhnpajkesaogrmnwzskmojslnzcgrpfieswiaihgymniohtftkodmktcnlkfw",
            "ur:provenance/lfaohdftihbkjoaagdadcadlwdlendsadknyynsabaadgrcxtdemtafrksoedttblnwekilasktdfdpdoxaeecsrckhgswltvwjelrwetelytlnllorkkitsnsyarshhbghy",
            "ur:provenance/lfaohdftdkgunnehhyuyuoeepfutguhsmdiadesgeceeueisamrpfmvtdaoectuoltahyldedmtnprwevdtpguospsdeqdlapkgofycydpldjzoxjsiystjntnihwdmohfox",
            "ur:provenance/lfaohdftlnnsesbseepaylvttbcsrdjefhnlnybahpvyjewfwlbwclgutkgtnyzefnptdradsbfsfeuohyiaployuybnfrbgkkisqzvatsisbziawsbazmonnemtsacmgwur",
            "ur:provenance/lfaohdftvymonsehvtyaspskvtrlgsrkqzzcrdecisvsgrchmhvshtmyrlnybsaszcctpakkleesmsvlmwmdwfemtahsiavwmdpfchgotitnwtrodshljscwbaieltvdbyyl",
            "ur:provenance/lfaohdftntnlhkiacedpldmecmcylehpwpchweprnybdframsffzatzmetdwnyfxswcpmttkurzsatfrjpbshdptiycwpfnlesisoyctbsrhmsiejyveoewdgdylredyestt",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfaohdfttokebznlpfgdjlhemhmevtztosmtoxwfbbmymtamltgtmdtahgheytskpmjpcffptldpecidgtvsqdhycnosashldacaaytncedeetidaxghfgftjnislbgosnpy",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftinhlpeoyettkvwetrnuyoespptjtsatnhndyldlbjlcnvwhsgendjefgvstyeerfjofwamclteahvwvwengooxtejszmswuecxlbhslrzmswhtkerswtdlynoyft",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdfttijoenjscfsnbkaogolngthkswmdtphgkitkptldwkfljndeutverevecwcplgglwtvedlmuayeoztjklursrdprntfsvyhpjtcwdpfylyoyvyhpimgaoxldswlo",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdfttegyylurwkcfaemyincabdwmvevdctinguonftyacnfddkckrsfgwzlbsbgwtksaeyosdwsstpgelrnnpsaekppdkomdgehnjodryapmnycmlgnldmaxaepllpdn",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftrstdmezckbjtqzuryajlkspydsbnvydweojsnypdhdjnlrbsykzefnmepramrdfnhnpajkesaogrmnwzskmojslnzcgrpfieswiaihgymniohtftkodmenkowlih",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftihbkjoaagdadcadlwdlendsadknyynsabaadgrcxtdemtafrksoedttblnwekilasktdfdpdoxaeecsrckhgswltvwjelrwetelytlnllorkkitsnsyazeasktkk",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftdkgunnehhyuyuoeepfutguhsmdiadesgeceeueisamrpfmvtdaoectuoltahyldedmtnprwevdtpguospsdeqdlapkgofycydpldjzoxjsiystjntnihpysteols",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftlnnsesbseepaylvttbcsrdjefhnlnybahpvyjewfwlbwclgutkgtnyzefnptdradsbfsfeuohyiaployuybnfrbgkkisqzvatsisbziawsbazmonnemtlsfxdrya",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftvymonsehvtyaspskvtrlgsrkqzzcrdecisvsgrchmhvshtmyrlnybsaszcctpakkleesmsvlmwmdwfemtahsiavwmdpfchgotitnwtrodshljscwbaieswprjyti",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdftntnlhkiacedpldmecmcylehpwpchweprnybdframsffzatzmetdwnyfxswcpmttkurzsatfrjpbshdptiycwpfnlesisoyctbsrhmsiejyveoewdgdylwkihhhyn",
        ];

        run_test(
            ProvenanceMarkResolution::Quartile,
            false,
            &expected_descriptions,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
            false
        );
    }

    #[test]
    fn test_quartile_with_info() {
        let expected_descriptions = [
            r#"ProvenanceMark(key: ce7c1599b0506f5f9091e0fca796a4f3, hash: 9a30e79e05124df8b6d8a5e56ef4f445, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 0, date: 2023-06-20T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 695dafa138cfe538bedba2c8a96ec2da, hash: a184b7a0837c9495aa0a2fc51bceddb2, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 1, date: 2023-06-21T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: d070367119cd0a0255864d59c695d857, hash: 9ee9d0eba328b046e20f8884b5ad85d0, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 2, date: 2023-06-22T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: d351f7dff419008f691d0bebe4e71f69, hash: d001bbf2d96422fa970e30cb9e738d5e, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 3, date: 2023-06-23T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: bfd291fd7e6eb4dff86f78ab260ce12c, hash: f60a16e729dbe2d4fb13a58318137b82, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 4, date: 2023-06-24T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 650a700450011d2fea8a9bc2249af6c2, hash: 01a56c232214996ae7954f462336f4eb, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 5, date: 2023-06-25T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 24539e315edbdc34b0dd5361956328ca, hash: 72521a94618be94689bb90ad409055d5, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 6, date: 2023-06-26T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 869c390f34b1f7e0d618ba6b3f999a0e, hash: 662c4a4de28a755ce451d3182f28e66f, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 7, date: 2023-06-27T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: e1929c31e0f8c8c5e0b74cbbb4fdba35, hash: b5d6537b8fc92b8bcf07e6ae427fb567, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 8, date: 2023-06-28T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 9d9959631c2d8991161a8a5bec17edb2, hash: 07bb057fca9dff0e9ee2423d9e146341, chainID: ce7c1599b0506f5f9091e0fca796a4f3, seq: 9, date: 2023-06-29T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
        ];

        let expected_bytewords = [
            "taco kite buzz nail puff good jowl hope math maze vast zest owls mint onyx wolf bulb many mint atom list gift mild tuna hang hope yurt silk poem jump chef flap cook leaf whiz game zoom tuna kiln zoom song news kiln next safe vast poem lava code dice exit iced apex gush frog fact join iris rock oval echo iced junk wolf arch gush unit ruby item tuna hang each need meow cook code warm keep days onyx tomb yank gala calm user undo vibe logo cash ruin very",
            "iron hill pose obey exit task view exit ruin ugly oboe soap part jolt saga twin horn duty loud lamb jowl cyan view huts game need jade frog vows tiny edge roof puff meow list bias dull duty meow hope code mint iron axis deli urge real jugs crux lamb huts liar zoom skew heat kite runs what girl buzz edge taco cash leaf epic twin vows road crux ruby even yank arch flew fact draw axis news user jugs visa list paid fizz king kite lion rust surf acid taxi",
            "taxi judo even jugs chef swan back also gyro lion gift hawk skew mild trip hang kiwi task part loud work fuel join dice unit vibe race vibe claw cusp lung girl legs yurt toys claw drum void undo unit love deli cook wall cost hope quad work jolt claw drop foxy lazy obey very help item gala claw wave paid leaf hard owls limp silk acid wave belt mild bulb limp bias luau veto undo urge item runs idea silk toil safe puma body onyx chef toys zest away foxy",
            "time gray yell user work chef able many iron cola bald warm vibe void cost iron guru open fact yoga cyan fund dark cook runs frog whiz lamb stub glow task saga half part inky work jazz main blue news tiny onyx time flap logo yawn beta webs judo door yoga poem navy calm lung nail drum apex draw onyx omit lazy exit lava wolf cash door luau inch poem eyes puff obey duty when visa grim liar gear foxy high keep glow slot news foxy inch part chef duty slot",
            "runs tied maze zinc knob jolt quiz user yoga jowl keys play days barn very draw echo jugs navy paid hard join liar bias yank zone fern maze purr atom road fern what knob omit veto inky safe girl exam flux atom acid barn memo drum rich view skew idea inch gray main into heat fact keno drum echo chef twin surf surf iced rock gear warm keno dull lung open puff dark zest purr bald flew door bias lazy figs puff cook owls help memo math webs road main edge",
            "inch back judo aqua good acid cola dull wand love need saga dark navy yawn saga beta acid gear crux tied exam tuna fair keys oboe diet tomb lion wave kiwi lava hang lung purr flew jazz cola wall saga bias beta gush lava leaf roof memo paid time lazy toil nail logo rock kiwi toys news yoga arch epic song sets idea view dark cusp high keys skew cash jazz taco crux surf peck navy kept user girl code monk horn vows math flew city cost duty epic yurt echo",
            "dark guru noon each holy ugly undo edge puff unit guru huts mild idea dice song epic edge urge iris atom ramp film vast data oboe cost undo list arch yell dice zone jowl pool warm warm good apex inky girl bulb keep pool road gift legs stub drop loud jazz onyx jugs inky slot join twin inch drum cola iris navy solo jade kite claw jump eyes exam yell what wasp jump flap hawk kite claw jade hard love note rock fact knob cook jazz jowl girl kept door noon",
            "lion news eyes bias edge puma yell vast tomb cats road jade fish nail navy beta help very jade wolf wall brew curl guru task gift navy zone fern part door acid many flux loud junk judo gyro road city wolf jolt roof noon drum able kite buzz toys iris buzz idea webs beta zoom open note mint free idle tomb limp girl gray tied drum keep item peck song idle kept mild iris knob keno mint ruin dice surf surf horn zoom drop able zone jump race yawn atom iced",
            "very memo news each vast yoga soap silk vast real gems rock quiz zinc road epic iris vows gear cash math vows heat many real navy bias axis zinc cost puma kick veto down very flap grim cook chef tomb gear keep pool keys rock gift claw wave taxi twin what redo days hill jugs claw beta idle limp scar loud gray brag exam hope monk warm silk numb limp jazz chef back wall grim leaf waxy jade fern pose wolf noon redo roof taco sets idle onyx fizz vibe iris",
            "next nail hawk idea code drop loud maze calm city love help wasp cash wave purr navy bald fair atom surf fizz aunt zoom exit draw navy flux skew cusp mint task miss crux idea love days play limp jade game foxy scar draw high purr flux duty bias rich miss idle jury vibe oboe wand good yell jolt void wave exit menu owls atom owls surf claw flap ugly webs buzz love tent veto song luau item jowl code peck vows void brag kick buzz frog kiwi drum undo logo",
        ];

        let expected_id_words = [
            "NAVY DUTY VOID NOON",
            "OBEY LIAR REAL NUMB",
            "NOON WALL TAXI WARM",
            "TAXI ACID ROCK WHIZ",
            "YAWN BACK CALM VOID",
            "ACID OPEN JAZZ CYAN",
            "JUMP GRIM CITY MEOW",
            "INKY DRAW GAME GIFT",
            "RACE TOMB GURU KING",
            "AUNT ROCK ARCH LAMB",
        ];

        let expected_bytemoji_ids = [
            "🎡 👍 🐔 🔔",
            "🪑 💕 🪭 🚪",
            "🔔 🦆 🧵 🐴",
            "🧵 😂 🛁 🐢",
            "🐙 🫠 🤢 🐔",
            "😂 📫 🌺 💀",
            "💧 🥯 🥳 🚨",
            "🌵 🫶 🥑 🌽",
            "🎀 👒 🍞 🌎",
            "😍 🛁 😋 🌐",
        ];

        let expected_urs = [
            "ur:provenance/lfaohdhgtokebznlpfgdjlhemhmevtztosmtoxwfbbmymtamltgtmdtahgheytskpmjpcffpcklfwzgezmtaknzmsgnsknntsevtpmlacedeetidaxghfgftjnisrkoleoidjkwfahghutryimtahgehndmwckcewmkpdsoxtbykgacmuruovelkgovtlf",
            "ur:provenance/lfaohdhginhlpeoyettkvwetrnuyoespptjtsatnhndyldlbjlcnvwhsgendjefgvstyeerfpfmwltbsdldymwhecemtinasdiuerljscxlbhslrzmswhtkerswtglbzeetochlfectnvsrdcxryenykahfwftdwasnsurjsvaltpdfzkgkelnssmnheqd",
            "ur:provenance/lfaohdhgtijoenjscfsnbkaogolngthkswmdtphgkitkptldwkfljndeutverevecwcplggllsyttscwdmvduoutledickwlctheqdwkjtcwdpfylyoyvyhpimgacwwepdlfhdoslpskadwebtmdbblpbsluvououeimrsiasktlsepabyoxcfternhfdi",
            "ur:provenance/lfaohdhgtegyylurwkcfaemyincabdwmvevdctinguonftyacnfddkckrsfgwzlbsbgwtksahfptiywkjzmnbenstyoxtefploynbawsjodryapmnycmlgnldmaxdwoxotlyetlawfchdrluihpmespfoydywnvagmlrgrfyhhkpgwstnsfyihpmhpjtox",
            "ur:provenance/lfaohdhgrstdmezckbjtqzuryajlkspydsbnvydweojsnypdhdjnlrbsykzefnmepramrdfnwtkbotvoiyseglemfxamadbnmodmrhvwswiaihgymniohtftkodmeocftnsfsfidrkgrwmkodllgonpfdkztprbdfwdrbslyfspfckoshpmomhwmyatihg",
            "ur:provenance/lfaohdhgihbkjoaagdadcadlwdlendsadknyynsabaadgrcxtdemtafrksoedttblnwekilahglgprfwjzcawlsabsbaghlalfrfmopdtelytlnllorkkitsnsyaahecsgssiavwdkcphhksswchjztocxsfpknykturglcemkhnvsmhfwcycteektosgd",
            "ur:provenance/lfaohdhgdkgunnehhyuyuoeepfutguhsmdiadesgeceeueisamrpfmvtdaoectuoltahyldezejlplwmwmgdaxiyglbbkpplrdgtlssbdpldjzoxjsiystjntnihdmcaisnysojekecwjpesemylwtwpjpfphkkecwjehdlenerkftkbckjzjlgeecjyzc",
            "ur:provenance/lfaohdhglnnsesbseepaylvttbcsrdjefhnlnybahpvyjewfwlbwclgutkgtnyzefnptdradmyfxldjkjogordcywfjtrfnndmaekebztsisbziawsbazmonnemtfeietblpglgytddmkpimpksgiektmdiskbkomtrndesfsfhnzmdpaezejppaqzhdad",
            "ur:provenance/lfaohdhgvymonsehvtyaspskvtrlgsrkqzzcrdecisvsgrchmhvshtmyrlnybsaszcctpakkvodnvyfpgmckcftbgrkpplksrkgtcwwetitnwtrodshljscwbaielpsrldgybgemhemkwmsknblpjzcfbkwlgmlfwyjefnpewfnnrorftossienbaordbd",
            "ur:provenance/lfaohdhgntnlhkiacedpldmecmcylehpwpchweprnybdframsffzatzmetdwnyfxswcpmttkmscxialedspylpjegefysrdwhhprfxdybsrhmsiejyveoewdgdyljtvdweetmuosamossfcwfpuywsbzlettvosgluimjlcepkvsvdbgkkbzfgkkjzlfwm",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgtokebznlpfgdjlhemhmevtztosmtoxwfbbmymtamltgtmdtahgheytskpmjpcffpcklfwzgezmtaknzmsgnsknntsevtpmlacedeetidaxghfgftjnisrkoleoidjkwfahghutryimtahgehndmwckcewmkpdsoxtbykgacmuruovedkpsptbt",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhginhlpeoyettkvwetrnuyoespptjtsatnhndyldlbjlcnvwhsgendjefgvstyeerfpfmwltbsdldymwhecemtinasdiuerljscxlbhslrzmswhtkerswtglbzeetochlfectnvsrdcxryenykahfwftdwasnsurjsvaltpdfzkgkelnjzktcmfn",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgtijoenjscfsnbkaogolngthkswmdtphgkitkptldwkfljndeutverevecwcplggllsyttscwdmvduoutledickwlctheqdwkjtcwdpfylyoyvyhpimgacwwepdlfhdoslpskadwebtmdbblpbsluvououeimrsiasktlsepabyoxcfkgflctpd",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgtegyylurwkcfaemyincabdwmvevdctinguonftyacnfddkckrsfgwzlbsbgwtksahfptiywkjzmnbenstyoxtefploynbawsjodryapmnycmlgnldmaxdwoxotlyetlawfchdrluihpmespfoydywnvagmlrgrfyhhkpgwstnsfyihahoedidn",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgrstdmezckbjtqzuryajlkspydsbnvydweojsnypdhdjnlrbsykzefnmepramrdfnwtkbotvoiyseglemfxamadbnmodmrhvwswiaihgymniohtftkodmeocftnsfsfidrkgrwmkodllgonpfdkztprbdfwdrbslyfspfckoshpmomhfxadnltp",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgihbkjoaagdadcadlwdlendsadknyynsabaadgrcxtdemtafrksoedttblnwekilahglgprfwjzcawlsabsbaghlalfrfmopdtelytlnllorkkitsnsyaahecsgssiavwdkcphhksswchjztocxsfpknykturglcemkhnvsmhfwcyctnsmnwyur",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgdkgunnehhyuyuoeepfutguhsmdiadesgeceeueisamrpfmvtdaoectuoltahyldezejlplwmwmgdaxiyglbbkpplrdgtlssbdpldjzoxjsiystjntnihdmcaisnysojekecwjpesemylwtwpjpfphkkecwjehdlenerkftkbckjzjlvosffsjp",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhglnnsesbseepaylvttbcsrdjefhnlnybahpvyjewfwlbwclgutkgtnyzefnptdradmyfxldjkjogordcywfjtrfnndmaekebztsisbziawsbazmonnemtfeietblpglgytddmkpimpksgiektmdiskbkomtrndesfsfhnzmdpaezejpcfgtbymn",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgvymonsehvtyaspskvtrlgsrkqzzcrdecisvsgrchmhvshtmyrlnybsaszcctpakkvodnvyfpgmckcftbgrkpplksrkgtcwwetitnwtrodshljscwbaielpsrldgybgemhemkwmsknblpjzcfbkwlgmlfwyjefnpewfnnrorftossieayzowflr",
            "https://example.com/validate?provenance=tngdgmgwhflfaohdhgntnlhkiacedpldmecmcylehpwpchweprnybdframsffzatzmetdwnyfxswcpmttkmscxialedspylpjegefysrdwhhprfxdybsrhmsiejyveoewdgdyljtvdweetmuosamossfcwfpuywsbzlettvosgluimjlcepkvsvdbgkkbzfgttmdsbie",
        ];

        run_test(
            ProvenanceMarkResolution::Quartile,
            true,
            &expected_descriptions,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
            false
        );
    }

    #[test]
    fn test_high() {
        let expected_descriptions = [
            r#"ProvenanceMark(key: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, hash: 57c0cf203d12f0c56e6c223178cfadb16cc6352a68c69bc1b1fe9712ebe7dd4f, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 0, date: 2023-06-20T12:00:00Z)"#,
            r#"ProvenanceMark(key: 695dafa138cfe538bedba2c8a96ec2dad070367119cd0a0255864d59c695d857, hash: 6b8fac78fd6471a352165ef4ee956ae6f9e0c75db8057b76f41a56c75c340844, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 1, date: 2023-06-21T12:00:00Z)"#,
            r#"ProvenanceMark(key: d351f7dff419008f691d0bebe4e71f69bfd291fd7e6eb4dff86f78ab260ce12c, hash: 40f74333405f7bf307a95e12d7cb3f451768c0b3fa01d8c01d283c8bfa052f74, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 2, date: 2023-06-22T12:00:00Z)"#,
            r#"ProvenanceMark(key: 650a700450011d2fea8a9bc2249af6c224539e315edbdc34b0dd5361956328ca, hash: fc1ac0c63f6bc44500642df576bd18517548266bfe140c24828bd2a1531e9086, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 3, date: 2023-06-23T12:00:00Z)"#,
            r#"ProvenanceMark(key: 869c390f34b1f7e0d618ba6b3f999a0ee1929c31e0f8c8c5e0b74cbbb4fdba35, hash: 45de8d39b233d4e6822d37fac96e6ab5939d0051d8c44e95558b0bfdc5da0e69, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 4, date: 2023-06-24T12:00:00Z)"#,
            r#"ProvenanceMark(key: 9d9959631c2d8991161a8a5bec17edb2cc519d3df4f241dc98a285963646294d, hash: bee8c0e9cce14f0cc8853e21e72078e82d361e358f419a3756a2524cdb99251a, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 5, date: 2023-06-25T12:00:00Z)"#,
            r#"ProvenanceMark(key: 1f1df300ca1ba7e6e192dfdc4debe4d643026c948ef84fc50fd81ef55a3fe95a, hash: 8e70979a721291e27dae9e518023fa14dd66d4166fe35d8588699f4d34118f51, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 6, date: 2023-06-26T12:00:00Z)"#,
            r#"ProvenanceMark(key: 10cf8fb2ae8719ead09a153aa193e05d7abf97501bac041942a1a502e7f5eeba, hash: 3f405e54bfe8a798979f42779299be8347b4615eb7ca4a59dbed6f2d02d53410, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 7, date: 2023-06-27T12:00:00Z)"#,
            r#"ProvenanceMark(key: 186d0597dfd56a71abf4e86004c822600e4c72f6e3cf9d2a0f0c03aff38bd48d, hash: 9519def537037cc8d481c7c160c205713f983be8408bcb4109d56d3f791e274a, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 8, date: 2023-06-28T12:00:00Z)"#,
            r#"ProvenanceMark(key: e079e69f4ad9ae5fd8ce003998741df6598920846424db654b41f341d50fc56b, hash: 6edb360d094910f556e2b389fa18b8af65a1aa4ff35bfb7e60eba3d1029d51f1, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 9, date: 2023-06-29T12:00:00Z)"#,
        ];

        let expected_bytewords = [
            "taco kite buzz nail puff good jowl hope math maze vast zest owls mint onyx wolf unit arch yurt flux down taco lava rich diet wave liar tomb hard jury taco blue kiwi king good kite gear game safe jade also blue cash puma silk high oval yawn jowl film list waxy fair fish back fizz oboe data kiln safe song claw zero when slot pose yank good vast urge undo heat play pose curl work cook iron yawn tied holy cash guru kite tent fizz swan puma loud exit eyes road zinc waxy half kiln figs cost axis game diet cook soap kept junk grim code pose part waxy",
            "iron hill pose obey exit task view exit ruin ugly oboe soap part jolt saga twin taxi judo even jugs chef swan back also gyro lion gift hawk skew mild trip hang fact junk cats many drum mint miss wall cyan paid visa note cusp vial fern gala half luck belt idea many oboe veto zinc exam aunt iron draw arch mint jury poem race ramp limp wall real huts math also oboe brag vast pool lazy ramp peck glow aunt time apex knob hill lamb back holy vast user idea fuel inch chef jade numb yawn beta jazz loud meow inch ramp edge swan iced paid apex iced paid",
            "time gray yell user work chef able many iron cola bald warm vibe void cost iron runs tied maze zinc knob jolt quiz user yoga jowl keys play days barn very draw jazz memo flux king cost oboe junk kick owls leaf cola road ramp obey math obey axis real down pose void mint idle vast arch hard code ruin high webs vast work zoom visa ruin barn inch flap puff limp grim puff vows buzz brag girl trip menu navy each vows fact diet gray each legs saga luck menu guru girl zinc warm fund loud junk trip mint surf wolf oboe blue item calm apex wall tiny horn",
            "inch back judo aqua good acid cola dull wand love need saga dark navy yawn saga dark guru noon each holy ugly undo edge puff unit guru huts mild idea dice song gyro cook free hawk cash gems cusp each hawk rich back veto bald limp free data numb list tent tomb road hope deli ruby hang rock zinc need knob navy calm buzz math frog exit edge paid yoga heat warm city scar door ruin girl guru note junk liar puff wave hard lamb cyan yoga ruby draw task days toys gyro rich gift quiz miss diet eyes judo huts away wall cash draw scar cash aqua jolt taco",
            "lion news eyes bias edge puma yell vast tomb cats road jade fish nail navy beta very memo news each vast yoga soap silk vast real gems rock quiz zinc road epic yawn item pose mild limp task jury pool guru quad void zinc cyan able wolf open echo math item owls fern code jowl unit deli pool body oval even taxi frog pose soap jolt foxy belt game surf holy pose twin liar skew frog wall trip game apex vial many user duty fair rich whiz play hawk visa good gift list jade memo iron mild surf cats join note gala even limp lion obey math menu code epic",
            "next nail hawk idea code drop loud maze calm city love help wasp cash wave purr surf gray next figs work whiz flap undo monk oboe limp mint even frog diet gift warm puff runs vast data oval yank guru high oboe jazz open rich huts plus work soap math kick bald kept tiny tiny atom horn drum data kite ugly axis keep inky menu kiln lamb drum help limp down hill yell cook barn ruby sets kite bald numb navy legs jump keno quiz blue judo rust kiwi fund taco cola kiln crux ramp free runs kiln deli atom quad song puff iris tuna skew hard hard fair silk",
            "cost cola wolf able song claw owls visa very memo user undo gift warm vibe tomb flux also jazz meow main yoga glow silk bias trip cook yank heat fish wall heat keys owls play obey twin echo vows rock part runs zinc silk lazy flew pool aunt into city ramp wand love visa lamb data zoom junk easy calm zone race even twin oval fizz view lava safe brew dull drum sets drop safe grim buzz city chef navy pose race gear news tent luau logo cyan ruby obey pool numb obey kiwi lava draw idea taco yell grim jump code gush down wave free wand need visa cash",
            "blue task many purr pool list chef wand taxi navy buzz fact obey menu vast hill kiln runs miss good claw plus aqua chef flew obey open also void yank waxy road code buzz grim exit quad junk idle pose zinc ramp wolf oval meow legs rust edge fizz heat apex many atom cyan hope luau hard yoga jugs yoga note into wasp memo love fish yank warm ugly toys kiln jowl barn liar judo vast foxy data sets frog junk many girl exit aunt fern trip axis omit vibe obey jugs mint join inch zinc flew apex brew what hawk dice wasp deli judo draw belt what inky ruin",
            "cats join arch miss user toil item jugs play work vows horn aqua soap cusp horn beta gems jump yawn vial task next door bias barn apex pose wolf luau tiny lung axis fact visa into oboe fuel song yoga vial flux hawk cost keep puma list quad free liar saga tomb zoom onyx zoom diet city redo huts yoga numb fair twin toil gear drop monk zinc keep even yoga toil high acid puff fuel flap tomb aunt fern stub trip trip figs idle menu lava jade nail gala diet game atom barn deli gyro hard hang ramp flap wasp mint tomb view hawk away apex lion fund into",
            "vast kick visa note game tuna pool hope trip taco able eyes monk jury cola yawn hawk loud crux liar idle dark ugly inch gear flap wolf flap toil bias silk jade gush kiwi zaps edge jade tomb vows what hawk zinc cola trip cusp redo city easy chef dark yoga bias zaps visa dice horn ruin flew gift also memo yell hawk dull claw sets whiz flew ruby pose belt mild skew runs beta beta work diet judo when tuna obey oval buzz tuna kiwi pool rock bald tuna back many calm jowl huts hard belt lion limp memo jazz idea claw epic days jazz news ruin oval veto",
        ];

        let expected_id_words = [
            "HANG RUST TASK CRUX",
            "JADE MANY PLUS KEYS",
            "FIZZ YELL FLUX ECHO",
            "ZEST CITY RUST SKEW",
            "FREE URGE LUNG EYES",
            "RUIN VOWS RUST WALL",
            "MAIN JUDO MISS NAVY",
            "FISH FIZZ HOLY GUSH",
            "MILD CHEF URGE YANK",
            "JOLT UGLY EVEN BELT",
        ];

        let expected_bytemoji_ids = [
            "🌭 🏀 🧶 😈",
            "🌹 🚗 📷 🌛",
            "🍌 🦑 🍓 👆",
            "🦭 🥳 🏀 💥",
            "🍒 🐻 🛑 🧠",
            "🎷 🐥 🏀 🦆",
            "🔺 💨 🛟 🎡",
            "🍋 🍌 🍜 🧀",
            "🚀 🤡 🐻 🪽",
            "🌻 🐹 🦷 😶",
        ];

        let expected_urs = [
            "ur:provenance/lfaxhdimtokebznlpfgdjlhemhmevtztosmtoxwfutahytfxdntolarhdtwelrtbhdjytobekikggdkegrgesejeaobechpaskhholynjlfmltwyfrfhbkfzoedaknsesgcwzownstpeykgdvtueuohtpypeclwkckinyntdhychgukettfzsnpaldetesrdzcwyhfknfsctasgedtckspktjkgmbtrfvtwm",
            "ur:provenance/lfaxhdiminhlpeoyettkvwetrnuyoespptjtsatntijoenjscfsnbkaogolngthkswmdtphgftjkcsmydmmtmswlcnpdvanecpvlfngahflkbtiamyoevozcematindwahmtjypmrerplpwlrlhsmhaooebgvtpllyrppkgwatteaxkbhllbbkhyvturiaflihcfjenbynbajzldmwihrpeesnidrhbednpm",
            "ur:provenance/lfaxhdimtegyylurwkcfaemyincabdwmvevdctinrstdmezckbjtqzuryajlkspydsbnvydwjzmofxkgctoejkkkoslfcardrpoymhoyasrldnpevdmtievtahhdcernhhwsvtwkzmvarnbnihfppflpgmpfvsbzbggltpmunyehvsftdtgyehlssalkmuguglzcwmfdldjktpmtsfwfoebeimcmbgzsntih",
            "ur:provenance/lfaxhdimihbkjoaagdadcadlwdlendsadknyynsadkgunnehhyuyuoeepfutguhsmdiadesggockfehkchgscpehhkrhbkvobdlpfedanblttttbrdhediryhgrkzcndkbnycmbzmhfgeteepdyahtwmcysrdrrnglgunejklrpfwehdlbcnyarydwtkdstsgorhgtqzmsdtesjohsaywlchdwsramchdisb",
            "ur:provenance/lfaxhdimlnnsesbseepaylvttbcsrdjefhnlnybavymonsehvtyaspskvtrlgsrkqzzcrdecynimpemdlptkjyplguqdvdzccnaewfoneomhimosfncejlutdiplbyolentifgpespjtfybtgesfhypetnlrswfgwltpgeaxvlmyurdyfrrhwzpyhkvagdgtltjemoinmdsfcsjnnegaenlplnoylylagody",
            "ur:provenance/lfaxhdimntnlhkiacedpldmecmcylehpwpchweprsfgyntfswkwzfpuomkoelpmtenfgdtgtwmpfrsvtdaolykguhhoejzonrhhspswkspmhkkbdkttytyamhndmdakeuyaskpiymuknlbdmhplpdnhlylckbnrysskebdnbnylsjpkoqzbejortkifdtocakncxrpferskndiamqdsgpfistaswgagrjprt",
            "ur:provenance/lfaxhdimctcawfaesgcwosvavymouruogtwmvetbfxaojzmwmnyagwskbstpckykhtfhwlhtksospyoytneovsrkptrszcsklyfwplatiocyrpwdlevalbdazmjkeycmzereentnolfzvwlasebwdldmssdpsegmbzcycfnyperegrnsttlulocnryoyplnboykiladwiatoylgmjpceghdnwefezolopebg",
            "ur:provenance/lfaxhdimbetkmyprplltcfwdtinybzftoymuvthlknrsmsgdcwpsaacffwoyonaovdykwyrdcebzgmetqdjkiepezcrpwfolmwlsrteefzhtaxmyamcnheluhdyajsyaneiowpmolefhykwmuytsknjlbnlrjovtfydassfgjkmygletatfntpasotveoyjsmtjnihzcfwaxbwwthkdewpdijodwcevldlrk",
            "ur:provenance/lfaxhdimcsjnahmsurtlimjspywkvshnaaspcphnbagsjpynvltkntdrbsbnaxpewflutylgasftvaiooeflsgyavlfxhkctkppaltqdfelrsatbzmoxzmdtcyrohsyanbfrtntlgrdpmkzckpenyatlhhadpfflfptbatfnsbtptpfsiemulajenlgadtgeambndigohdhgrpfpwpmttbvwhkaybgmdadid",
            "ur:provenance/lfaxhdimvtkkvanegetaplhetptoaeesmkjycaynhkldcxlriedkuyihgrfpwffptlbsskjeghkizseejetbvswthkzccatpcprocyeycfdkyabszsvadehnrnfwgtaomoylhkdlcwsswzfwrypebtmdswrsbabawkdtjowntaoyolbztakiplrkbdtabkmycmjlhshdbtlnlpmojziacwecdsjzlgpmwsvd",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimtokebznlpfgdjlhemhmevtztosmtoxwfutahytfxdntolarhdtwelrtbhdjytobekikggdkegrgesejeaobechpaskhholynjlfmltwyfrfhbkfzoedaknsesgcwzownstpeykgdvtueuohtpypeclwkckinyntdhychgukettfzsnpaldetesrdzcwyhfknfsctasgedtckspktjkgmbyvlglks",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdiminhlpeoyettkvwetrnuyoespptjtsatntijoenjscfsnbkaogolngthkswmdtphgftjkcsmydmmtmswlcnpdvanecpvlfngahflkbtiamyoevozcematindwahmtjypmrerplpwlrlhsmhaooebgvtpllyrppkgwatteaxkbhllbbkhyvturiaflihcfjenbynbajzldmwihrpeesnidongwlpfm",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimtegyylurwkcfaemyincabdwmvevdctinrstdmezckbjtqzuryajlkspydsbnvydwjzmofxkgctoejkkkoslfcardrpoymhoyasrldnpevdmtievtahhdcernhhwsvtwkzmvarnbnihfppflpgmpfvsbzbggltpmunyehvsftdtgyehlssalkmuguglzcwmfdldjktpmtsfwfoebeimcmbaoneoyn",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimihbkjoaagdadcadlwdlendsadknyynsadkgunnehhyuyuoeepfutguhsmdiadesggockfehkchgscpehhkrhbkvobdlpfedanblttttbrdhediryhgrkzcndkbnycmbzmhfgeteepdyahtwmcysrdrrnglgunejklrpfwehdlbcnyarydwtkdstsgorhgtqzmsdtesjohsaywlchdwsrcyfdldhd",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimlnnsesbseepaylvttbcsrdjefhnlnybavymonsehvtyaspskvtrlgsrkqzzcrdecynimpemdlptkjyplguqdvdzccnaewfoneomhimosfncejlutdiplbyolentifgpespjtfybtgesfhypetnlrswfgwltpgeaxvlmyurdyfrrhwzpyhkvagdgtltjemoinmdsfcsjnnegaenlplnoynturzoot",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimntnlhkiacedpldmecmcylehpwpchweprsfgyntfswkwzfpuomkoelpmtenfgdtgtwmpfrsvtdaolykguhhoejzonrhhspswkspmhkkbdkttytyamhndmdakeuyaskpiymuknlbdmhplpdnhlylckbnrysskebdnbnylsjpkoqzbejortkifdtocakncxrpferskndiamqdsgpfistaswgobbuogu",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimctcawfaesgcwosvavymouruogtwmvetbfxaojzmwmnyagwskbstpckykhtfhwlhtksospyoytneovsrkptrszcsklyfwplatiocyrpwdlevalbdazmjkeycmzereentnolfzvwlasebwdldmssdpsegmbzcycfnyperegrnsttlulocnryoyplnboykiladwiatoylgmjpceghdnwefevdtsadly",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimbetkmyprplltcfwdtinybzftoymuvthlknrsmsgdcwpsaacffwoyonaovdykwyrdcebzgmetqdjkiepezcrpwfolmwlsrteefzhtaxmyamcnheluhdyajsyaneiowpmolefhykwmuytsknjlbnlrjovtfydassfgjkmygletatfntpasotveoyjsmtjnihzcfwaxbwwthkdewpdijodwaerflyde",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimcsjnahmsurtlimjspywkvshnaaspcphnbagsjpynvltkntdrbsbnaxpewflutylgasftvaiooeflsgyavlfxhkctkppaltqdfelrsatbzmoxzmdtcyrohsyanbfrtntlgrdpmkzckpenyatlhhadpfflfptbatfnsbtptpfsiemulajenlgadtgeambndigohdhgrpfpwpmttbvwhkaybasgpewn",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdimvtkkvanegetaplhetptoaeesmkjycaynhkldcxlriedkuyihgrfpwffptlbsskjeghkizseejetbvswthkzccatpcprocyeycfdkyabszsvadehnrnfwgtaomoylhkdlcwsswzfwrypebtmdswrsbabawkdtjowntaoyolbztakiplrkbdtabkmycmjlhshdbtlnlpmojziacwecdsjzmewzfpjy",
        ];

        run_test(
            ProvenanceMarkResolution::High,
            false,
            &expected_descriptions,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
            false
        );
    }

    #[test]
    fn test_high_with_info() {
        let expected_descriptions = [
            r#"ProvenanceMark(key: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, hash: dfa31e8bc3b90a8e52b3f859a03c8a32e1f5618daa00c764d84686fefbd22945, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 0, date: 2023-06-20T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 695dafa138cfe538bedba2c8a96ec2dad070367119cd0a0255864d59c695d857, hash: a24aad4462e7da702f3a543489436464c488fdfb60858a1e5db3ce3fd664fd18, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 1, date: 2023-06-21T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: d351f7dff419008f691d0bebe4e71f69bfd291fd7e6eb4dff86f78ab260ce12c, hash: 82e1cbc60a1cc6ace839d4d15b7a2e0534121ef6d2b9035e200a1e6e5071c25e, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 2, date: 2023-06-22T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 650a700450011d2fea8a9bc2249af6c224539e315edbdc34b0dd5361956328ca, hash: f6c497c9dca3b7d31840ccd78fb793fa85151ceefbefb4a16c7211647bd11e82, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 3, date: 2023-06-23T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 869c390f34b1f7e0d618ba6b3f999a0ee1929c31e0f8c8c5e0b74cbbb4fdba35, hash: 73581631a3723f433b92df85a42a8226291ff3ee0df20a429ee8dced1761265a, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 4, date: 2023-06-24T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 9d9959631c2d8991161a8a5bec17edb2cc519d3df4f241dc98a285963646294d, hash: cc67f5784eb524980c77905b6f05078af93073272e3b67cbf8994bb29e183d75, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 5, date: 2023-06-25T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 1f1df300ca1ba7e6e192dfdc4debe4d643026c948ef84fc50fd81ef55a3fe95a, hash: 3c03066b0e97a8dd4ce76f6a88ea294e2181a59c90eec04b6979c8b282d279c1, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 6, date: 2023-06-26T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 10cf8fb2ae8719ead09a153aa193e05d7abf97501bac041942a1a502e7f5eeba, hash: e433b5d98c96b3d59d9ebd2f23d462a67732fd6dcc0abee46a6d4d7fbe8af16b, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 7, date: 2023-06-27T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: 186d0597dfd56a71abf4e86004c822600e4c72f6e3cf9d2a0f0c03aff38bd48d, hash: 2b80e81d801656240f91ac3ec4e9ad8e2ca1aaa9efeef00504ff64d1a3ede62e, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 8, date: 2023-06-28T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
            r#"ProvenanceMark(key: e079e69f4ad9ae5fd8ce003998741df6598920846424db654b41f341d50fc56b, hash: 7a8e1652acd45d4231cdf60db4063b95fee4bb5ec72bead5a63eff50a1384f48, chainID: ce7c1599b0506f5f9091e0fca796a4f3dd05f9432bce80b929ed84d65874ce10, seq: 9, date: 2023-06-29T12:00:00Z, info: "Lorem ipsum sit dolor amet.")"#,
        ];

        let expected_bytewords = [
            "taco kite buzz nail puff good jowl hope math maze vast zest owls mint onyx wolf unit arch yurt flux down taco lava rich diet wave liar tomb hard jury taco blue kiwi king good kite gear game safe jade also blue cash puma silk high oval yawn jowl film list waxy fair fish back fizz oboe data kiln safe song claw zero when glow surf dark zero cook keep days body miss judo zero news skew navy tent gray time dark aunt ugly brew lion maze bulb vast lava dice half wave ugly oboe judo figs cost axis game diet cook soap kept junk grim iris luau vibe obey whiz numb draw memo gear luck vast jade even slot grim inky pose tent gyro slot grim song away fern duty limp lion skew easy unit hang yurt diet",
            "iron hill pose obey exit task view exit ruin ugly oboe soap part jolt saga twin taxi judo even jugs chef swan back also gyro lion gift hawk skew mild trip hang fact junk cats many drum mint miss wall cyan paid visa note cusp vial fern gala half luck belt idea many oboe veto zinc exam aunt iron draw arch mint jury poem kite junk liar toil dice veto fair tent user film wand jolt visa horn onyx swan fact rock eyes trip limp zoom zero even gala keno zero runs webs gala noon zest yawn beta jazz loud meow inch ramp edge swan iced aunt math ugly list poem drum inch game hang kite tuna acid bulb days zero zero grim unit onyx luck jury ruby waxy iris zinc warm half zaps taco maze horn gush toil",
            "time gray yell user work chef able many iron cola bald warm vibe void cost iron runs tied maze zinc knob jolt quiz user yoga jowl keys play days barn very draw jazz memo flux king cost oboe junk kick owls leaf cola road ramp obey math obey axis real down pose void mint idle vast arch hard code ruin high webs vast work figs what even yurt dull also belt twin ruby crux iced tomb noon zoom solo time rich gear even lamb acid wall wand cola zoom pool puma ramp vibe loud atom iced loud junk trip mint surf wolf oboe blue item calm omit bulb beta nail luck duty join omit song maze twin kept wolf road judo open poem wand next apex drop back monk king dice eyes fact junk main void claw play lazy",
            "inch back judo aqua good acid cola dull wand love need saga dark navy yawn saga dark guru noon each holy ugly undo edge puff unit guru huts mild idea dice song gyro cook free hawk cash gems cusp each hawk rich back veto bald limp free data numb list tent tomb road hope deli ruby hang rock zinc need knob navy calm buzz navy monk jowl fair gear duty diet kiwi also void stub news real hawk bulb trip jury wave toys unit kiln trip fizz exit saga even view brag kiwi keno scar puff miss diet eyes judo huts away wall cash draw scar jump epic work judo runs peck ruby ugly wolf help item calm poem scar hope flew fizz axis urge horn glow limp onyx slot body flap duty dull fact very list data into",
            "lion news eyes bias edge puma yell vast tomb cats road jade fish nail navy beta very memo news each vast yoga soap silk vast real gems rock quiz zinc road epic yawn item pose mild limp task jury pool guru quad void zinc cyan able wolf open echo math item owls fern code jowl unit deli pool body oval even taxi frog pose zone vows user arch help lung race back idea fair drum eyes liar news oboe math hawk belt draw many waxy many ramp kite memo limp list hill gyro taxi road heat mild surf cats join note gala even limp lion obey surf wall also vows void item aqua taxi high vibe away foxy toys iris kite quiz owls draw claw luck easy iris view very aqua flew memo poem numb tuna chef unit idle",
            "next nail hawk idea code drop loud maze calm city love help wasp cash wave purr surf gray next figs work whiz flap undo monk oboe limp mint even frog diet gift warm puff runs vast data oval yank guru high oboe jazz open rich huts plus work soap math kick bald kept tiny tiny atom horn drum data kite ugly axis keep inky very yank game runs tuna tent fizz solo echo wasp oboe slot gems hawk jury saga girl limp cost idle buzz item lung fern time junk toys vial fish obey pool door runs kiln deli atom quad song puff iris tuna skew jump aqua purr next wand fund solo work flew purr obey brag bald menu flew horn bias duty echo body noon puma buzz jowl item kept cusp taxi grim twin luck exit hill",
            "cost cola wolf able song claw owls visa very memo user undo gift warm vibe tomb flux also jazz meow main yoga glow silk bias trip cook yank heat fish wall heat keys owls play obey twin echo vows rock part runs zinc silk lazy flew pool aunt into city ramp wand love visa lamb data zoom junk easy calm zone race even twin bulb echo jury jugs ruby mint calm body yank idle duty iron cola time song rust guru grim fact calm drum lion buzz wave high puma yurt hope cash ruin keno roof idea taco yell grim jump code gush down wave free frog zone real hard jugs jazz kite wolf fuel hawk kept taco gala inky yoga taxi grim taxi city vibe flew visa item fund fern main eyes gyro crux runs yurt flap lazy",
            "blue task many purr pool list chef wand taxi navy buzz fact obey menu vast hill kiln runs miss good claw plus aqua chef flew obey open also void yank waxy road code buzz grim exit quad junk idle pose zinc ramp wolf oval meow legs rust edge fizz heat apex many atom cyan hope luau hard yoga jugs yoga note into wasp memo gray gems cook inky vows part jolt cusp atom limp many redo yank iris cats idea flux axis tied bald kite zest draw quiz brag idle legs cyan door easy numb lion flew apex brew what hawk dice wasp deli judo draw fair hawk horn frog part safe loud hang tiny next undo stub quiz calm omit runs luck poem menu cola knob need wand flux bias undo taco jump bald slot cola junk heat",
            "cats join arch miss user toil item jugs play work vows horn aqua soap cusp horn beta gems jump yawn vial task next door bias barn apex pose wolf luau tiny lung axis fact visa into oboe fuel song yoga vial flux hawk cost keep puma list quad free liar saga tomb zoom onyx zoom diet city redo huts yoga numb fair twin toil yank quiz pool buzz saga cyan tied eyes list body ugly redo view zinc pose scar trip very gala kite stub yawn rock dull meow idea crux onyx undo zoom visa each hard hang ramp flap wasp mint tomb view hawk away claw fish pose wave need fact girl race barn gear data vast drop toys urge toil need buzz wave jump help aunt peck time tiny lava cats cola bias code cook aqua visa",
            "vast kick visa note game tuna pool hope trip taco able eyes monk jury cola yawn hawk loud crux liar idle dark ugly inch gear flap wolf flap toil bias silk jade gush kiwi zaps edge jade tomb vows what hawk zinc cola trip cusp redo city easy chef dark yoga bias zaps visa dice horn ruin flew gift also memo yell hawk dull bias maze tied cola cats easy fizz cusp obey math gear love road exam wolf stub flew vibe real aqua wave belt runs blue swan barn half beta race song lamb very belt lion limp memo jazz idea claw epic days jazz luau horn race cook jowl claw loud taxi vial paid rock tomb brag drop atom puff pool list deli yell luck ugly wolf deli cyan legs noon cook brew hard inch need wall",
        ];

        let expected_id_words = [
            "USER OMIT COOK LUAU",
            "OBOE GAME POEM FOXY",
            "LEAF VERY STUB SKEW",
            "YAWN SETS MISS SOLO",
            "JUNK HARD CALM EACH",
            "SURF INTO YANK KEYS",
            "FERN APEX ATOM JADE",
            "VIBE ECHO RACE TUNA",
            "DOWN LAVA VOWS COLA",
            "KILN MAIN CALM GRIM",
        ];

        let expected_bytemoji_ids = [
            "🐼 💌 🙃 🔷",
            "🎈 🥑 ⏰ 🫐",
            "💘 🐯 👗 💥",
            "🐙 ✨ 🛟 👖",
            "💦 🍔 🤢 👎",
            "👔 🌱 🪽 🌛",
            "🦶 😉 😎 🌹",
            "🐷 👆 🎀 🐶",
            "😿 💛 🐥 🤑",
            "🌙 🔺 🤢 🥯",
        ];

        let expected_urs = [
            "ur:provenance/lfaxhdlttokebznlpfgdjlhemhmevtztosmtoxwfutahytfxdntolarhdtwelrtbhdjytobekikggdkegrgesejeaobechpaskhholynjlfmltwyfrfhbkfzoedaknsesgcwzowngwsfdkzockkpdsbymsjozonsswnyttgytedkatuybwlnmebbvtladehfweuyoejofsctasgedtckspktjkgmisluveoywznbdwmogrlkvtjeenstgmiypettgostgmsgayfndylplnsweyvelkspgt",
            "ur:provenance/lfaxhdltinhlpeoyettkvwetrnuyoespptjtsatntijoenjscfsnbkaogolngthkswmdtphgftjkcsmydmmtmswlcnpdvanecpvlfngahflkbtiamyoevozcematindwahmtjypmkejklrtldevofrtturfmwdjtvahnoxsnftrkestplpzmzoengakozorswsgannztynbajzldmwihrpeesnidatmhuyltpmdmihgehgketaadbbdszozogmutoxlkjyrywyiszcwmhfzstopdrkihpa",
            "ur:provenance/lfaxhdlttegyylurwkcfaemyincabdwmvevdctinrstdmezckbjtqzuryajlkspydsbnvydwjzmofxkgctoejkkkoslfcardrpoymhoyasrldnpevdmtievtahhdcernhhwsvtwkfswtenytdlaobttnrycxidtbnnzmsoterhgrenlbadwlwdcazmplparpveldamidldjktpmtsfwfoebeimcmotbbbanllkdyjnotsgmetnktwfrdjoonpmwdntaxdpbkmkkgdeesftjkmnuertnyvw",
            "ur:provenance/lfaxhdltihbkjoaagdadcadlwdlendsadknyynsadkgunnehhyuyuoeepfutguhsmdiadesggockfehkchgscpehhkrhbkvobdlpfedanblttttbrdhediryhgrkzcndkbnycmbznymkjlfrgrdydtkiaovdsbnsrlhkbbtpjywetsutkntpfzetsaenvwbgkikosrpfmsdtesjohsaywlchdwsrjpecwkjorspkryuywfhpimcmpmsrhefwfzasuehngwlpoxstbyfpdydlfttphhbbax",
            "ur:provenance/lfaxhdltlnnsesbseepaylvttbcsrdjefhnlnybavymonsehvtyaspskvtrlgsrkqzzcrdecynimpemdlptkjyplguqdvdzccnaewfoneomhimosfncejlutdiplbyolentifgpezevsurahhplgrebkiafrdmeslrnsoemhhkbtdwmywymyrpkemolplthlgotirdhtmdsfcsjnnegaenlplnoysfwlaovsvdimaatihhveayfytsiskeqzosdwcwlkeyisvwvyaafwmopmnbvtsawpae",
            "ur:provenance/lfaxhdltntnlhkiacedpldmecmcylehpwpchweprsfgyntfswkwzfpuomkoelpmtenfgdtgtwmpfrsvtdaolykguhhoejzonrhhspswkspmhkkbdkttytyamhndmdakeuyaskpiyvyykgerstattfzsoeowpoestgshkjysagllpctiebzimlgfntejktsvlfhoypldrrskndiamqdsgpfistaswjpaaprntwdfdsowkfwproybgbdmufwhnbsdyeobynnpabzjlimktcptigmvlhgases",
            "ur:provenance/lfaxhdltctcawfaesgcwosvavymouruogtwmvetbfxaojzmwmnyagwskbstpckykhtfhwlhtksospyoytneovsrkptrszcsklyfwplatiocyrpwdlevalbdazmjkeycmzereentnbbeojyjsrymtcmbyykiedyincatesgrtgugmftcmdmlnbzwehhpaythechrnkorfiatoylgmjpceghdnwefefgzerlhdjsjzkewfflhkkttogaiyyatigmticyvefwvaimfdfnmnesgocxlncpjovw",
            "ur:provenance/lfaxhdltbetkmyprplltcfwdtinybzftoymuvthlknrsmsgdcwpsaacffwoyonaovdykwyrdcebzgmetqdjkiepezcrpwfolmwlsrteefzhtaxmyamcnheluhdyajsyaneiowpmogygsckiyvsptjtcpamlpmyroykiscsiafxastdbdkeztdwqzbgielscndreynblnfwaxbwwthkdewpdijodwfrhkhnfgptseldhgtyntuosbqzcmotrslkpmmucakbndwdfxbsuotojpbdzeswfwfm",
            "ur:provenance/lfaxhdltcsjnahmsurtlimjspywkvshnaaspcphnbagsjpynvltkntdrbsbnaxpewflutylgasftvaiooeflsgyavlfxhkctkppaltqdfelrsatbzmoxzmdtcyrohsyanbfrtntlykqzplbzsacntdesltbyuyrovwzcpesrtpvygakesbynrkdlmwiacxoxuozmvaehhdhgrpfpwpmttbvwhkaycwfhpewendftglrebngrdavtdptsuetlndbzwejphpatpktetylacscabsdaskeclf",
            "ur:provenance/lfaxhdltvtkkvanegetaplhetptoaeesmkjycaynhkldcxlriedkuyihgrfpwffptlbsskjeghkizseejetbvswthkzccatpcprocyeycfdkyabszsvadehnrnfwgtaomoylhkdlbsmetdcacseyfzcpoymhgrlerdemwfsbfwverlaawebtrsbesnbnhfbaresglbvybtlnlpmojziacwecdsjzluhnreckjlcwldtivlpdrktbbgdpampfplltdiyllkuywfdicnlsnnckbwhsrnpklg",
        ];

        let expected_urls = [
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdlttokebznlpfgdjlhemhmevtztosmtoxwfutahytfxdntolarhdtwelrtbhdjytobekikggdkegrgesejeaobechpaskhholynjlfmltwyfrfhbkfzoedaknsesgcwzowngwsfdkzockkpdsbymsjozonsswnyttgytedkatuybwlnmebbvtladehfweuyoejofsctasgedtckspktjkgmisluveoywznbdwmogrlkvtjeenstgmiypettgostgmsgayfndylplnsweyylbwvsht",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltinhlpeoyettkvwetrnuyoespptjtsatntijoenjscfsnbkaogolngthkswmdtphgftjkcsmydmmtmswlcnpdvanecpvlfngahflkbtiamyoevozcematindwahmtjypmkejklrtldevofrtturfmwdjtvahnoxsnftrkestplpzmzoengakozorswsgannztynbajzldmwihrpeesnidatmhuyltpmdmihgehgketaadbbdszozogmutoxlkjyrywyiszcwmhfzstorkdkfeol",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdlttegyylurwkcfaemyincabdwmvevdctinrstdmezckbjtqzuryajlkspydsbnvydwjzmofxkgctoejkkkoslfcardrpoymhoyasrldnpevdmtievtahhdcernhhwsvtwkfswtenytdlaobttnrycxidtbnnzmsoterhgrenlbadwlwdcazmplparpveldamidldjktpmtsfwfoebeimcmotbbbanllkdyjnotsgmetnktwfrdjoonpmwdntaxdpbkmkkgdeesftjkmnsnherdwz",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltihbkjoaagdadcadlwdlendsadknyynsadkgunnehhyuyuoeepfutguhsmdiadesggockfehkchgscpehhkrhbkvobdlpfedanblttttbrdhediryhgrkzcndkbnycmbznymkjlfrgrdydtkiaovdsbnsrlhkbbtpjywetsutkntpfzetsaenvwbgkikosrpfmsdtesjohsaywlchdwsrjpecwkjorspkryuywfhpimcmpmsrhefwfzasuehngwlpoxstbyfpdydlftsbsreebb",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltlnnsesbseepaylvttbcsrdjefhnlnybavymonsehvtyaspskvtrlgsrkqzzcrdecynimpemdlptkjyplguqdvdzccnaewfoneomhimosfncejlutdiplbyolentifgpezevsurahhplgrebkiafrdmeslrnsoemhhkbtdwmywymyrpkemolplthlgotirdhtmdsfcsjnnegaenlplnoysfwlaovsvdimaatihhveayfytsiskeqzosdwcwlkeyisvwvyaafwmopmnbwfhlsfch",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltntnlhkiacedpldmecmcylehpwpchweprsfgyntfswkwzfpuomkoelpmtenfgdtgtwmpfrsvtdaolykguhhoejzonrhhspswkspmhkkbdkttytyamhndmdakeuyaskpiyvyykgerstattfzsoeowpoestgshkjysagllpctiebzimlgfntejktsvlfhoypldrrskndiamqdsgpfistaswjpaaprntwdfdsowkfwproybgbdmufwhnbsdyeobynnpabzjlimktcptigmwtspdtdm",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltctcawfaesgcwosvavymouruogtwmvetbfxaojzmwmnyagwskbstpckykhtfhwlhtksospyoytneovsrkptrszcsklyfwplatiocyrpwdlevalbdazmjkeycmzereentnbbeojyjsrymtcmbyykiedyincatesgrtgugmftcmdmlnbzwehhpaythechrnkorfiatoylgmjpceghdnwefefgzerlhdjsjzkewfflhkkttogaiyyatigmticyvefwvaimfdfnmnesgocxmdrygdwz",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltbetkmyprplltcfwdtinybzftoymuvthlknrsmsgdcwpsaacffwoyonaovdykwyrdcebzgmetqdjkiepezcrpwfolmwlsrteefzhtaxmyamcnheluhdyajsyaneiowpmogygsckiyvsptjtcpamlpmyroykiscsiafxastdbdkeztdwqzbgielscndreynblnfwaxbwwthkdewpdijodwfrhkhnfgptseldhgtyntuosbqzcmotrslkpmmucakbndwdfxbsuotojpbdwehkiddt",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltcsjnahmsurtlimjspywkvshnaaspcphnbagsjpynvltkntdrbsbnaxpewflutylgasftvaiooeflsgyavlfxhkctkppaltqdfelrsatbzmoxzmdtcyrohsyanbfrtntlykqzplbzsacntdesltbyuyrovwzcpesrtpvygakesbynrkdlmwiacxoxuozmvaehhdhgrpfpwpmttbvwhkaycwfhpewendftglrebngrdavtdptsuetlndbzwejphpatpktetylacscabsenhtbzmd",
            "https://example.com/validate?provenance=tngdgmgwhflfaxhdltvtkkvanegetaplhetptoaeesmkjycaynhkldcxlriedkuyihgrfpwffptlbsskjeghkizseejetbvswthkzccatpcprocyeycfdkyabszsvadehnrnfwgtaomoylhkdlbsmetdcacseyfzcpoymhgrlerdemwfsbfwverlaawebtrsbesnbnhfbaresglbvybtlnlpmojziacwecdsjzluhnreckjlcwldtivlpdrktbbgdpampfplltdiyllkuywfdicnlsnnckbwjpclleny",
        ];

        run_test(
            ProvenanceMarkResolution::High,
            true,
            &expected_descriptions,
            &expected_bytewords,
            &expected_id_words,
            &expected_bytemoji_ids,
            &expected_urs,
            &expected_urls,
            false
        );
    }
}
