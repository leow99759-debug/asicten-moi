//! «Что нового сегодня» (SPEC §8 news): RSS headlines + CoinGecko prices → a spoken digest,
//! summarised by Gemini when the user set a key, else read as plain headlines. Pure, tested.

use crate::info::plural;

/// (topic, Google News query; `None` = the «В мире» section). Russian, Ukrainian edition.
pub const TOPICS: [(&str, Option<&str>); 4] = [
    ("В мире", None),
    ("Криптовалюта", Some("криптовалюта OR биткоин")),
    (
        "Рынки",
        Some("фондовый рынок OR S&P 500 OR золото OR нефть"),
    ),
    (
        "Технологии и ИИ",
        Some("искусственный интеллект OR OpenAI OR Anthropic OR нейросеть"),
    ),
];

const GNEWS: &str = "https://news.google.com/rss";
const EDITION: &str = "hl=ru&gl=UA&ceid=UA:ru";

/// Bitcoin, Ether and gold (tether-gold ≈ 1 troy ounce) in dollars only (user: no €/₴
/// conversions) with 24 h change.
pub const PRICES_URL: &str = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin,ethereum,tether-gold&vs_currencies=usd&include_24hr_change=true";

/// Feed URL for a topic (percent-encoded: curl.exe args are not UTF-8 safe on Windows).
pub fn feed_url(query: Option<&str>) -> String {
    match query {
        None => format!("{GNEWS}/headlines/section/topic/WORLD?{EDITION}"),
        Some(q) => format!(
            "{GNEWS}/search?q={}&{EDITION}",
            pct(&format!("{q} when:1d"))
        ),
    }
}

fn pct(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            b' ' => "+".into(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// First `n` item titles of an RSS feed, entities decoded, « - Source» suffix dropped.
pub fn titles(xml: &str, n: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in xml.split("<item>").skip(1) {
        let Some(raw) = between(item, "<title>", "</title>") else {
            continue;
        };
        let raw = raw
            .trim()
            .trim_start_matches("<![CDATA[")
            .trim_end_matches("]]>");
        let mut t = decode(raw);
        // Google News: «Title - Source», the source also sits in <source>
        let source = between(item, "<source", "</source>")
            .and_then(|x| x.split_once('>'))
            .map(|(_, name)| format!(" - {}", decode(name.trim())));
        match source {
            Some(src) if t.ends_with(&src) => t.truncate(t.len() - src.len()),
            _ => {
                if let Some(i) = t.rfind(" - ") {
                    if t[i..].chars().count() < 50 {
                        t.truncate(i);
                    }
                }
            }
        }
        let t = t.trim().to_owned();
        if !t.is_empty() && !out.contains(&t) {
            out.push(t);
        }
        if out.len() == n {
            break;
        }
    }
    out
}

fn between<'a>(s: &'a str, a: &str, b: &str) -> Option<&'a str> {
    let start = s.find(a)? + a.len();
    let end = s[start..].find(b)? + start;
    Some(&s[start..end])
}

fn decode(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// Prices as data lines for the LLM and as the spoken market part of the plain digest.
pub fn prices(json: &str) -> Vec<String> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(json) else {
        return vec![];
    };
    [
        ("bitcoin", "Биткоин"),
        ("ethereum", "Эфир"),
        ("tether-gold", "Золото за унцию"),
    ]
    .iter()
    .filter_map(|(id, name)| {
        let c = &v[*id];
        let usd = c["usd"].as_f64()?;
        let change = c["usd_24h_change"].as_f64().unwrap_or(0.0);
        let dollars = usd.round() as i64;
        Some(format!(
            "{name} — {dollars} {}, {} за сутки",
            plural(dollars, "доллар", "доллара", "долларов"),
            change_phrase(change)
        ))
    })
    .collect()
}

fn change_phrase(p: f64) -> String {
    let r = (p * 10.0).round() / 10.0;
    if r == 0.0 {
        return "без изменений".into();
    }
    let sign = if r > 0.0 { "плюс" } else { "минус" };
    format!(
        "{sign} {} процента",
        format!("{:.1}", r.abs()).replace('.', ",")
    )
}

/// Task for Gemini: a short spoken digest in Jarvis's manner.
pub fn prompt(sections: &[(String, Vec<String>)], prices: &[String]) -> String {
    let mut p = String::from(
        "Ты Джарвис, ироничный британский дворецкий-ассистент. Составь устную сводку новостей \
         за сегодня для хозяина (обращайся «сэр»). По-русски, 6–10 коротких предложений, \
         около 800 символов. Порядок: мировая политика, криптовалюта, фондовый рынок и золото, \
         технологии и ИИ. Связывай причины и следствия, если они видны из данных. Используй \
         только факты из данных ниже, ничего не выдумывай, пропусти пустые разделы. \
         Все суммы и цены называй только в долларах, не пересчитывай в гривны, евро или \
         другие валюты. Без markdown, списков, \
         эмодзи и ссылок: текст будет озвучен.\n\nКотировки:\n",
    );
    for l in prices {
        p.push_str(&format!("- {l}\n"));
    }
    for (topic, heads) in sections {
        p.push_str(&format!("\n{topic}:\n"));
        for h in heads {
            p.push_str(&format!("- {h}\n"));
        }
    }
    p
}

/// No LLM key (or every key failed): two headlines per topic + prices.
pub fn plain(sections: &[(String, Vec<String>)], prices: &[String]) -> String {
    let mut out = String::from("Вот что нового, сэр.");
    for (topic, heads) in sections {
        let heads: Vec<&str> = heads
            .iter()
            .take(2)
            .map(|h| h.trim_end_matches('.'))
            .collect();
        let nums: Vec<String> = match topic.as_str() {
            "Криптовалюта" => prices.iter().take(2).cloned().collect(),
            "Рынки" => prices.iter().skip(2).cloned().collect(),
            _ => vec![],
        };
        if heads.is_empty() && nums.is_empty() {
            continue;
        }
        out.push_str(&format!(" {topic}: "));
        let parts: Vec<&str> = nums.iter().map(String::as_str).chain(heads).collect();
        out.push_str(&parts.join(". "));
        out.push('.');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const RSS: &str = r#"<rss><channel><title>В мире - Google Новости</title>
      <item><title>Биткоин упал на 5% - РБК</title></item>
      <item><title><![CDATA[Трамп &quot;заявил&quot; о пошлинах - bbc.com]]></title></item>
      <item><title>Биткоин упал на 5% - Українські Національні Новини (УНН)</title><source url="https://unn.ua">Українські Національні Новини (УНН)</source></item>
      <item><title>Третья</title></item></channel></rss>"#;

    const PRICES: &str = r#"{"bitcoin":{"usd":83497.2,"usd_24h_change":-1.78,"eur":73489,"eur_24h_change":-1.2,"uah":3747102,"uah_24h_change":-1.6},
      "tether-gold":{"usd":4147.97,"usd_24h_change":0.01,"eur":3650.7,"uah":186148}}"#;

    #[test]
    fn titles_skip_channel_decode_and_dedupe() {
        assert_eq!(
            titles(RSS, 2),
            vec!["Биткоин упал на 5%", "Трамп \"заявил\" о пошлинах"]
        );
        assert_eq!(titles(RSS, 9).len(), 3);
        assert!(titles("<html>blocked</html>", 3).is_empty());
    }

    #[test]
    fn urls_are_ascii() {
        let u = feed_url(TOPICS[1].1);
        assert!(u.is_ascii() && u.contains("q=%D0%BA") && u.contains("when%3A1d"));
        assert!(feed_url(None).contains("topic/WORLD"));
    }

    #[test]
    fn price_lines() {
        let p = prices(PRICES);
        assert_eq!(
            p[0],
            "Биткоин — 83497 долларов, минус 1,8 процента за сутки"
        );
        assert!(
            p[1].starts_with("Золото за унцию — 4148 долларов")
                && p[1].ends_with("без изменений за сутки")
        );
        assert!(prices("oops").is_empty());
    }

    #[test]
    fn plain_digest_reads_prices_and_headlines() {
        let s = vec![
            ("В мире".to_owned(), titles(RSS, 3)),
            ("Криптовалюта".to_owned(), vec![]),
            ("Технологии и ИИ".to_owned(), vec![]),
        ];
        let d = plain(&s, &prices(PRICES));
        assert!(d.contains("В мире: Биткоин упал на 5%. Трамп"));
        assert!(d.contains("Криптовалюта: Биткоин — 83497 долларов, минус 1,8"));
        assert!(!d.contains("Технологии"));
        assert!(prompt(&s, &prices(PRICES)).contains("- Третья"));
    }
}
