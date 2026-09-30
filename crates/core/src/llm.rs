//! Online chat models for the news digest and free questions («Джарвис, почему небо голубое»).
//! Pure part: provider catalog, request bodies, reply parsing. HTTP lives in jarvis-win.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One model the user added in Settings → ИИ. The list order is the priority.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct AiSlot {
    /// [`Provider::id`].
    pub provider: String,
    pub key: String,
    /// Empty = the provider's default model(s).
    pub model: String,
    /// Only for `custom`: an OpenAI-compatible chat/completions URL (Ollama, LM Studio…).
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wire {
    /// POST {model, messages} → choices[0].message.content (most providers).
    OpenAi,
    Anthropic,
    Gemini,
}

pub struct Provider {
    pub id: &'static str,
    pub name: &'static str,
    pub url: &'static str,
    /// Tried in order when the slot has no model (the first may be retired by then).
    pub models: &'static [&'static str],
    pub wire: Wire,
}

/// Popular chat models with an API key. `custom` = any OpenAI-compatible server.
pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: "openai",
        name: "ChatGPT (OpenAI)",
        url: "https://api.openai.com/v1/chat/completions",
        models: &["gpt-4.1-mini", "gpt-4o-mini"],
        wire: Wire::OpenAi,
    },
    Provider {
        id: "anthropic",
        name: "Claude (Anthropic)",
        url: "https://api.anthropic.com/v1/messages",
        models: &["claude-haiku-4-5", "claude-3-5-haiku-latest"],
        wire: Wire::Anthropic,
    },
    Provider {
        id: "gemini",
        name: "Gemini (Google)",
        url: "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent",
        models: &["gemini-2.5-flash-lite", "gemini-flash-lite-latest"],
        wire: Wire::Gemini,
    },
    Provider {
        id: "deepseek",
        name: "DeepSeek",
        url: "https://api.deepseek.com/chat/completions",
        models: &["deepseek-chat"],
        wire: Wire::OpenAi,
    },
    Provider {
        id: "qwen",
        name: "Qwen (Alibaba)",
        url: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1/chat/completions",
        models: &["qwen-plus", "qwen-turbo"],
        wire: Wire::OpenAi,
    },
    Provider {
        id: "kimi",
        name: "Kimi (Moonshot)",
        url: "https://api.moonshot.ai/v1/chat/completions",
        models: &["kimi-k2-turbo-preview", "moonshot-v1-8k"],
        wire: Wire::OpenAi,
    },
    Provider {
        id: "grok",
        name: "Grok (xAI)",
        url: "https://api.x.ai/v1/chat/completions",
        models: &["grok-3-mini", "grok-4-fast-non-reasoning"],
        wire: Wire::OpenAi,
    },
    Provider {
        id: "mistral",
        name: "Mistral",
        url: "https://api.mistral.ai/v1/chat/completions",
        models: &["mistral-small-latest"],
        wire: Wire::OpenAi,
    },
    Provider {
        id: "perplexity",
        name: "Perplexity (с поиском в интернете)",
        url: "https://api.perplexity.ai/chat/completions",
        models: &["sonar"],
        wire: Wire::OpenAi,
    },
    Provider {
        id: "glm",
        name: "GLM (Zhipu / Z.ai)",
        url: "https://api.z.ai/api/paas/v4/chat/completions",
        models: &["glm-4.5-flash", "glm-4-flash"],
        wire: Wire::OpenAi,
    },
    Provider {
        id: "groq",
        name: "Groq (Llama, бесплатно)",
        url: "https://api.groq.com/openai/v1/chat/completions",
        models: &["llama-3.3-70b-versatile"],
        wire: Wire::OpenAi,
    },
    Provider {
        id: "openrouter",
        name: "OpenRouter (все модели одним ключом)",
        url: "https://openrouter.ai/api/v1/chat/completions",
        models: &["openrouter/auto"],
        wire: Wire::OpenAi,
    },
    Provider {
        id: "custom",
        name: "Свой сервер (OpenAI-совместимый: Ollama, LM Studio…)",
        url: "",
        models: &[],
        wire: Wire::OpenAi,
    },
];

pub fn provider(id: &str) -> Option<&'static Provider> {
    PROVIDERS.iter().find(|p| p.id == id)
}

/// A ready-to-send call: URL, secret header lines, JSON body.
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub url: String,
    pub headers: String,
    pub body: String,
    pub wire: Wire,
}

impl AiSlot {
    /// Usable: a known provider with a key (a custom server may run without one).
    pub fn ready(&self) -> bool {
        match self.provider.as_str() {
            "custom" => !self.url.trim().is_empty(),
            id => provider(id).is_some() && !self.key.trim().is_empty(),
        }
    }

    /// Models to try, in order: the user's, then the provider defaults (a typo or a retired
    /// model still gets an answer).
    pub fn models(&self) -> Vec<String> {
        let mut out: Vec<String> = Some(self.model.trim())
            .filter(|m| !m.is_empty())
            .map(str::to_owned)
            .into_iter()
            .collect();
        for m in provider(&self.provider).map_or(&[][..], |p| p.models) {
            if !out.iter().any(|o| o == m) {
                out.push((*m).to_owned());
            }
        }
        out
    }

    pub fn request(&self, model: &str, system: &str, user: &str) -> Option<Request> {
        let p = provider(&self.provider)?;
        let key = self.key.trim();
        let bearer = if key.is_empty() {
            String::new()
        } else {
            format!("Authorization: Bearer {key}")
        };
        let messages = |sys_inline: bool| {
            let mut m = Vec::new();
            if sys_inline && !system.is_empty() {
                m.push(serde_json::json!({"role": "system", "content": system}));
            }
            m.push(serde_json::json!({"role": "user", "content": user}));
            m
        };
        Some(match p.wire {
            Wire::OpenAi => Request {
                url: if p.id == "custom" {
                    self.url.trim().to_owned()
                } else {
                    p.url.to_owned()
                },
                headers: bearer,
                body: {
                    let mut b = serde_json::json!({"model": model, "messages": messages(true)});
                    // OpenAI's newer models reject max_tokens; everyone else knows only max_tokens
                    let cap = if p.id == "openai" {
                        "max_completion_tokens"
                    } else {
                        "max_tokens"
                    };
                    b[cap] = MAX_TOKENS.into();
                    b.to_string()
                },
                wire: p.wire,
            },
            Wire::Anthropic => Request {
                url: p.url.to_owned(),
                headers: format!("x-api-key: {key}\nanthropic-version: 2023-06-01"),
                body: serde_json::json!({
                    "model": model, "max_tokens": MAX_TOKENS, "system": system,
                    "messages": messages(false),
                })
                .to_string(),
                wire: p.wire,
            },
            Wire::Gemini => Request {
                url: p.url.replace("{model}", model),
                headers: format!("x-goog-api-key: {key}"),
                body: {
                    let mut b = serde_json::json!({
                        "contents": [{"parts": [{"text": user}]}],
                        "generationConfig": {"temperature": 0.4, "maxOutputTokens": MAX_TOKENS},
                    });
                    if !system.is_empty() {
                        b["system_instruction"] = serde_json::json!({"parts": [{"text": system}]});
                    }
                    b.to_string()
                },
                wire: p.wire,
            },
        })
    }
}

/// Enough for an ~800-char spoken digest; answers are asked to be short anyway.
const MAX_TOKENS: u32 = 900;

/// Reply text (cleaned for the voice), or the provider's error message.
pub fn reply_text(wire: Wire, json: &str) -> Result<String, String> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|_| short(json, "не JSON-ответ"))?;
    let text: String = match wire {
        Wire::OpenAi => v["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        Wire::Anthropic => v["content"]
            .as_array()
            .map(|ps| ps.iter().filter_map(|p| p["text"].as_str()).collect())
            .unwrap_or_default(),
        Wire::Gemini => v["candidates"][0]["content"]["parts"]
            .as_array()
            .map(|ps| ps.iter().filter_map(|p| p["text"].as_str()).collect())
            .unwrap_or_default(),
    };
    if !text.trim().is_empty() {
        return Ok(for_voice(&text));
    }
    let err = &v["error"];
    Err(err["message"]
        .as_str()
        .or_else(|| err.as_str())
        .or_else(|| v["message"].as_str())
        .map_or_else(|| short(json, "пустой ответ"), str::to_owned))
}

fn short(s: &str, fallback: &str) -> String {
    let s: String = s.trim().chars().take(160).collect();
    if s.is_empty() {
        fallback.to_owned()
    } else {
        s
    }
}

/// Drop markdown and reasoning traces the model may still add.
pub fn for_voice(text: &str) -> String {
    let text = match (text.find("<think>"), text.find("</think>")) {
        (Some(a), Some(b)) if a < b => format!("{}{}", &text[..a], &text[b + 8..]),
        _ => text.to_owned(),
    };
    text.lines()
        .map(|l| {
            l.trim()
                .trim_start_matches(['-', '•', '#', ' ', '>'])
                .replace(['*', '`'], "")
        })
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// System prompt for free questions.
pub const ASK_SYSTEM: &str = "Ты Джарвис, ироничный британский дворецкий-ассистент из фильмов \
    «Железный человек». Отвечай по-русски, обращайся «сэр». Коротко: 1–3 предложения, до 300 \
    символов, если не просят подробнее. Без markdown, списков, эмодзи и ссылок: ответ будет \
    озвучен. Числа и единицы пиши так, чтобы их было легко произнести. Если не знаешь свежих \
    данных, честно скажи об этом.";

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(provider: &str) -> AiSlot {
        AiSlot {
            provider: provider.into(),
            key: "k".into(),
            ..Default::default()
        }
    }

    #[test]
    fn every_provider_builds_a_request() {
        for p in PROVIDERS.iter().filter(|p| p.id != "custom") {
            let s = slot(p.id);
            assert!(s.ready() && !s.models().is_empty(), "{}", p.id);
            let r = s.request(&s.models()[0], "sys", "вопрос").expect("req");
            assert!(
                r.url.starts_with("https://") && !r.url.contains("{model}"),
                "{}",
                p.id
            );
            assert!(r.body.contains("вопрос"), "{}", p.id);
        }
        let mut c = slot("custom");
        c.key.clear();
        assert!(!c.ready());
        c.url = "http://localhost:11434/v1/chat/completions".into();
        c.model = "qwen2.5".into();
        let r = c.request("qwen2.5", "", "q").expect("custom");
        assert!(c.ready() && r.headers.is_empty() && r.url.contains("11434"));
    }

    #[test]
    fn wire_specifics() {
        let a = slot("anthropic").request("m", "sys", "u").expect("a");
        assert!(a.headers.contains("x-api-key: k") && a.body.contains("\"system\":\"sys\""));
        let o = slot("openai").request("m", "sys", "u").expect("o");
        assert!(o.body.contains("max_completion_tokens") && o.headers == "Authorization: Bearer k");
        assert!(slot("deepseek")
            .request("m", "", "u")
            .expect("d")
            .body
            .contains("\"max_tokens\""));
        let g = slot("gemini").request("gemini-x", "sys", "u").expect("g");
        assert!(g.url.contains("models/gemini-x:") && g.body.contains("system_instruction"));
    }

    #[test]
    fn replies_and_errors() {
        let o = r#"{"choices":[{"message":{"content":"<think>hm</think>**Сэр**, небо голубое.\n- Рэлей."}}]}"#;
        assert_eq!(
            reply_text(Wire::OpenAi, o).expect("o"),
            "Сэр, небо голубое. Рэлей."
        );
        let a = r#"{"content":[{"type":"text","text":"Да, сэр."}]}"#;
        assert_eq!(reply_text(Wire::Anthropic, a).expect("a"), "Да, сэр.");
        let e = r#"{"error":{"message":"Incorrect API key provided"}}"#;
        assert_eq!(
            reply_text(Wire::OpenAi, e).expect_err("e"),
            "Incorrect API key provided"
        );
        let ae = r#"{"type":"error","error":{"type":"authentication_error","message":"invalid x-api-key"}}"#;
        assert_eq!(
            reply_text(Wire::Anthropic, ae).expect_err("ae"),
            "invalid x-api-key"
        );
        assert!(reply_text(Wire::OpenAi, "<html>502</html>").is_err());
    }
}
