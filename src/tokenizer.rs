use std::str::FromStr;
use tokenizers::Tokenizer;

pub struct TokenizationResult {
    pub ids: Vec<u32>,
    pub attention_mask: Vec<u32>,
    pub offsets: Vec<(usize, usize)>,
    pub tokens: Vec<String>,
}

pub struct MaskaraTokenizer {
    tokenizer: Tokenizer,
}

impl MaskaraTokenizer {
    pub fn new() -> Self {
        let json_str = include_str!("../assets/tokenizer.json");
        let tokenizer = Tokenizer::from_str(json_str).expect("Failed to load embedded tokenizer");
        Self { tokenizer }
    }

    pub fn encode(&self, text: &str, add_special_tokens: bool) -> anyhow::Result<TokenizationResult> {
        // encode the text.
        let encoding = self.tokenizer.encode(text, add_special_tokens).map_err(|e| anyhow::anyhow!(e))?;
        let ids = encoding.get_ids().to_vec();
        let attention_mask = encoding.get_attention_mask().to_vec();
        let offsets = encoding.get_offsets()
            .iter()
            .map(|&(start, end)| (start, end))
            .collect::<Vec<_>>();
        let tokens = encoding.get_tokens().to_vec();
        Ok(TokenizationResult {
            ids,
            attention_mask,
            offsets,
            tokens,
        })
    }
}
