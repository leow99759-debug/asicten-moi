//! Voice level 3 (SPEC §6.3): the built-in Windows voice (WinRT SpeechSynthesizer).
//! Returns WAV bytes so playback goes through our player (volume, stop, orb level).

use windows::core::HSTRING;
use windows::Media::SpeechSynthesis::SpeechSynthesizer;
use windows::Storage::Streams::DataReader;

fn e(x: windows::core::Error) -> String {
    x.message().to_string()
}

/// Synthesize `text` with a Russian voice if one is installed. `speed` 1.0 = normal.
pub fn synth_wav(text: &str, speed: f32) -> Result<Vec<u8>, String> {
    let s = SpeechSynthesizer::new().map_err(e)?;
    if let Ok(voices) = SpeechSynthesizer::AllVoices() {
        let ru = (0..voices.Size().unwrap_or(0))
            .filter_map(|i| voices.GetAt(i).ok())
            .find(|v| v.Language().is_ok_and(|l| l.to_string().starts_with("ru")));
        if let Some(v) = ru {
            s.SetVoice(&v).map_err(e)?;
        }
    }
    if let Ok(o) = s.Options() {
        let _ = o.SetSpeakingRate(f64::from(speed.clamp(0.5, 3.0)));
    }
    let stream = s
        .SynthesizeTextToStreamAsync(&HSTRING::from(text))
        .and_then(|op| op.join())
        .map_err(e)?;
    let size = u32::try_from(stream.Size().map_err(e)?).map_err(|x| x.to_string())?;
    let reader =
        DataReader::CreateDataReader(&stream.GetInputStreamAt(0).map_err(e)?).map_err(e)?;
    reader.LoadAsync(size).and_then(|op| op.join()).map_err(e)?;
    let mut buf = vec![0u8; size as usize];
    reader.ReadBytes(&mut buf).map_err(e)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    /// CI runners may lack speech voices; when synthesis works, the WAV must decode.
    #[test]
    fn windows_voice_wav_decodes() {
        match super::synth_wav("Проверка связи", 1.0) {
            Ok(wav) => {
                let (s, rate) =
                    jarvis_core::voice::decode_wav(std::io::Cursor::new(wav)).expect("decode");
                assert!(rate >= 8_000 && s.len() > rate as usize / 4);
            }
            Err(e) => eprintln!("no Windows voice here: {e}"),
        }
    }
}
