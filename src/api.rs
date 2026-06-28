use crate::engine::MaskaraEngine;
use crate::tokenizer::MaskaraTokenizer;
use crate::decode::{decode_bio_spans, TokenPrediction};
use crate::window::merge_overlapping_entities;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity: String,
    pub value: String,
    pub start: usize,
    pub end: usize,
    pub score: f32,
}

#[derive(Debug, Clone, Default)]
pub struct MaskOptions {
    pub entities: Option<Vec<String>>,
    pub threshold: Option<f32>,
    pub placeholder: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RedactOptions {
    pub entities: Option<Vec<String>>,
    pub threshold: Option<f32>,
}

pub static TOKENIZER: Lazy<MaskaraTokenizer> = Lazy::new(|| MaskaraTokenizer::new());
pub static ENGINE: Lazy<MaskaraEngine> = Lazy::new(|| MaskaraEngine::new().expect("Failed to initialize MaskaraEngine"));

pub fn detect(text: &str) -> anyhow::Result<Vec<Entity>> {
    if text.is_empty() {
        return Ok(Vec::new());
    }

    // Tokenize WITHOUT special tokens first
    let encoding = TOKENIZER.encode(text, false)?;
    let ids = encoding.ids;
    let offsets = encoding.offsets;

    let cls_id = 101;
    let sep_id = 102;
    let window_size = 254; // Leaves room for CLS and SEP to make total length 256
    let overlap = 128;
    
    let mut all_spans = Vec::new();

    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut start_idx = 0;
    while start_idx < ids.len() {
        let end_idx = std::cmp::min(start_idx + window_size, ids.len());
        let chunk_ids = &ids[start_idx..end_idx];
        
        // Prepend [CLS] and append [SEP]
        let mut input_ids = Vec::with_capacity(chunk_ids.len() + 2);
        input_ids.push(cls_id);
        input_ids.extend_from_slice(chunk_ids);
        input_ids.push(sep_id);

        let token_type_ids = vec![0; input_ids.len()];

        // Run prediction
        let probs = ENGINE.predict(&input_ids, &token_type_ids)?;
        let probs_data = probs.to_vec2::<f32>()?;

        // Parse token predictions
        let mut token_predictions = Vec::with_capacity(chunk_ids.len());
        for i in 0..chunk_ids.len() {
            let row = &probs_data[i + 1]; // +1 to skip CLS
            let (argmax, &max_val) = row.iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap();

            let (tok_start, tok_end) = offsets[start_idx + i];
            token_predictions.push(TokenPrediction {
                label_id: argmax,
                score: max_val,
                start: tok_start,
                end: tok_end,
            });
        }

        // Decode BIO spans for this window
        let window_spans = decode_bio_spans(&token_predictions, text);
        all_spans.extend(window_spans);

        if end_idx == ids.len() {
            break;
        }
        start_idx += window_size - overlap;
    }

    // Merge overlapping spans from different windows using highest score
    let merged_spans = merge_overlapping_entities(all_spans);
    Ok(merged_spans)
}

pub fn mask(text: &str, options: Option<MaskOptions>) -> String {
    let opts = options.unwrap_or_default();
    let threshold = opts.threshold.unwrap_or(0.5);
    let placeholder_template = opts.placeholder.as_deref().unwrap_or("[{entity}]");

    let mut entities = match detect(text) {
        Ok(e) => e,
        Err(_) => return text.to_string(),
    };

    entities.retain(|e| {
        if e.score < threshold {
            return false;
        }
        if let Some(ref allowed) = opts.entities {
            return allowed.contains(&e.entity);
        }
        true
    });

    // Sort from back to front to safely replace characters without altering preceding indexes
    entities.sort_by_key(|e| std::cmp::Reverse(e.start));

    let mut masked_text = text.to_string();
    for ent in entities {
        let ph = placeholder_template.replace("{entity}", &ent.entity);
        if ent.start <= masked_text.len() && ent.end <= masked_text.len() {
            masked_text.replace_range(ent.start..ent.end, &ph);
        }
    }
    masked_text
}

pub fn redact(text: &str, options: Option<RedactOptions>) -> String {
    let opts = options.unwrap_or_default();
    let threshold = opts.threshold.unwrap_or(0.5);

    let mut entities = match detect(text) {
        Ok(e) => e,
        Err(_) => return text.to_string(),
    };

    entities.retain(|e| {
        if e.score < threshold {
            return false;
        }
        if let Some(ref allowed) = opts.entities {
            return allowed.contains(&e.entity);
        }
        true
    });

    // Sort from back to front to safely replace characters without altering preceding indexes
    entities.sort_by_key(|e| std::cmp::Reverse(e.start));

    let mut redacted_text = text.to_string();
    for ent in entities {
        if ent.start <= redacted_text.len() && ent.end <= redacted_text.len() {
            redacted_text.replace_range(ent.start..ent.end, "");
        }
    }
    redacted_text
}

pub fn detect_batch(texts: Vec<String>) -> anyhow::Result<Vec<Vec<Entity>>> {
    let mut results = Vec::with_capacity(texts.len());
    for text in texts {
        results.push(detect(&text)?);
    }
    Ok(results)
}
