use bc_ur::{UR, URDecodable, UREncodable};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    ProvenanceMark,
    util::{deserialize_ur, serialize_ur},
};

#[derive(Serialize, Clone)]
pub struct ProvenanceMarkInfo {
    #[serde(
        serialize_with = "serialize_ur",
        deserialize_with = "deserialize_ur"
    )]
    ur: UR,

    bytewords: String,

    bytemoji: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    comment: String,

    mark: ProvenanceMark,
}

impl<'de> Deserialize<'de> for ProvenanceMarkInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ProvenanceMarkInfoHelper {
            #[serde(
                serialize_with = "serialize_ur",
                deserialize_with = "deserialize_ur"
            )]
            ur: UR,
            bytewords: String,
            bytemoji: String,
            #[serde(default)]
            comment: String,
            // We deserialize mark but ignore it in favor of parsing from UR
            #[serde(default)]
            #[allow(dead_code)]
            mark: Option<serde_json::Value>,
        }

        let helper = ProvenanceMarkInfoHelper::deserialize(deserializer)?;

        // Deserialize the mark from the UR to ensure date_bytes and seq_bytes
        // match what was originally generated
        let mark = ProvenanceMark::from_ur(&helper.ur)
            .map_err(serde::de::Error::custom)?;

        Ok(ProvenanceMarkInfo {
            ur: helper.ur,
            bytewords: helper.bytewords,
            bytemoji: helper.bytemoji,
            comment: helper.comment,
            mark,
        })
    }
}

impl ProvenanceMarkInfo {
    pub fn new(mark: ProvenanceMark, comment: impl Into<String>) -> Self {
        let ur = mark.ur();
        let bytewords = mark.bytewords_identifier(true);
        let bytemoji = mark.bytemoji_identifier(true);
        let comment = comment.into();
        Self { mark, ur, bytewords, bytemoji, comment }
    }

    pub fn mark(&self) -> &ProvenanceMark { &self.mark }

    pub fn ur(&self) -> &UR { &self.ur }

    pub fn bytewords(&self) -> &str { &self.bytewords }

    pub fn bytemoji(&self) -> &str { &self.bytemoji }

    pub fn comment(&self) -> &str { &self.comment }

    /*
    Example of a markdown summary:

    ```markdown
    ---

    #### ur:provenance/lfaohdftzmdwbgkguywftyghfmdpprsagapschryvtwtrnrpwzoxclcllgvsaycabbkefphnaxassgssylinrlchzoztfreywkaoinlyhysbsasraaftiahngytsyljzvthh

    2025-01-17T01:12:33Z

    `🅟 WAVE JUDO LIAR FIGS`

    🅟 🐝 💨 💕 🍎

    Genesis mark.

    ```

    The `####` header minus special characters can often be used as a URL slug on
    places like GitHub and other markdown renderers:

    `https://github.com/...#urprovenancelfaohdftbstsfpiylnsfguiepthynnvwplaokpinzsctttbgskfxmtldtddtrheysgiocpgyhsetnlmwatrtvyrywmamiygstsmnkilrkinylygliantynpemssscygeoehs`
    */

    pub fn markdown_summary(&self) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push("---".to_string());

        lines.push("".to_string());
        lines.push(format!("{}", self.mark.date()));

        lines.push("".to_string());
        lines.push(format!("#### {}", self.ur));

        lines.push("".to_string());
        lines.push(format!("#### `{}`", self.bytewords));

        lines.push("".to_string());
        lines.push(self.bytemoji.clone().to_string());

        lines.push("".to_string());
        if !self.comment.is_empty() {
            lines.push(self.comment.clone());
            lines.push("".to_string());
        }

        lines.join("\n")
    }
}
