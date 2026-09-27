//! Listening modes (SPEC §2.2): prefix / silent / mic off, switched by UI, hotkey or voice.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::{Config, ModePhrases};
use crate::text::normalize as norm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ModeCommand {
    PrefixOn,
    PrefixOff,
    SilentOn,
    SilentOff,
    MicOn,
    MicOff,
}

impl ModeCommand {
    pub fn apply(self, cfg: &mut Config) {
        match self {
            Self::PrefixOn => cfg.prefix_mode = true,
            Self::PrefixOff => cfg.prefix_mode = false,
            Self::SilentOn => cfg.silent_mode = true,
            Self::SilentOff => cfg.silent_mode = false,
            Self::MicOn => cfg.mic_enabled = true,
            Self::MicOff => cfg.mic_enabled = false,
        }
    }
}

/// Voice phrase → mode switch. Matches when the utterance contains the configured phrase
/// (so «джарвис, перейди в тихий режим» works too).
pub fn from_phrase(text: &str, p: &ModePhrases) -> Option<ModeCommand> {
    let t = format!(" {} ", norm(text));
    [
        (&p.prefix_off, ModeCommand::PrefixOff),
        (&p.prefix_on, ModeCommand::PrefixOn),
        (&p.silent_off, ModeCommand::SilentOff),
        (&p.silent_on, ModeCommand::SilentOn),
        (&p.mic_off, ModeCommand::MicOff),
    ]
    .into_iter()
    .find(|(phrase, _)| {
        let ph = norm(phrase);
        !ph.is_empty() && t.contains(&format!(" {ph} "))
    })
    .map(|(_, cmd)| cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phrases_map_to_commands() {
        let p = ModePhrases::default();
        let cases = [
            ("Перейди в режим префикса", Some(ModeCommand::PrefixOn)),
            ("выключи режим префикса", Some(ModeCommand::PrefixOff)),
            (
                "Джарвис, перейди в тихий режим!",
                Some(ModeCommand::SilentOn),
            ),
            ("выключи тихий режим", Some(ModeCommand::SilentOff)),
            ("хватит слушать", Some(ModeCommand::MicOff)),
            ("включи музыку", None),
            ("перейди в тихий", None),
        ];
        for (text, want) in cases {
            assert_eq!(from_phrase(text, &p), want, "{text}");
        }
    }

    #[test]
    fn apply_changes_config() {
        let mut c = Config::default();
        ModeCommand::PrefixOff.apply(&mut c);
        ModeCommand::SilentOn.apply(&mut c);
        ModeCommand::MicOff.apply(&mut c);
        assert!(!c.prefix_mode && c.silent_mode && !c.mic_enabled);
    }
}
