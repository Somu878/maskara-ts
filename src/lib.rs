pub mod api;
pub mod decode;
pub mod engine;
pub mod tokenizer;
pub mod window;

use napi_derive::napi;
use api::MaskOptions as CoreMaskOptions;
use api::RedactOptions as CoreRedactOptions;

#[napi(object)]
pub struct Entity {
    pub entity: String,
    pub value: String,
    pub start: u32,
    pub end: u32,
    pub score: f64,
}

#[napi(object)]
pub struct MaskOptions {
    pub entities: Option<Vec<String>>,
    pub threshold: Option<f64>,
    pub placeholder: Option<String>,
}

#[napi(object)]
pub struct RedactOptions {
    pub entities: Option<Vec<String>>,
    pub threshold: Option<f64>,
}

#[napi]
pub fn detect(text: String) -> napi::Result<Vec<Entity>> {
    let core_entities = api::detect(&text)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        
    let entities = core_entities.into_iter().map(|e| Entity {
        entity: e.entity,
        value: e.value,
        start: e.start as u32,
        end: e.end as u32,
        score: e.score as f64,
    }).collect();
    
    Ok(entities)
}

#[napi]
pub fn mask(text: String, options: Option<MaskOptions>) -> String {
    let core_options = options.map(|o| CoreMaskOptions {
        entities: o.entities,
        threshold: o.threshold.map(|t| t as f32),
        placeholder: o.placeholder,
    });
    api::mask(&text, core_options)
}

#[napi]
pub fn redact(text: String, options: Option<RedactOptions>) -> String {
    let core_options = options.map(|o| CoreRedactOptions {
        entities: o.entities,
        threshold: o.threshold.map(|t| t as f32),
    });
    api::redact(&text, core_options)
}

#[napi]
pub fn detect_batch(texts: Vec<String>) -> napi::Result<Vec<Vec<Entity>>> {
    let core_results = api::detect_batch(texts)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        
    let results = core_results.into_iter().map(|list| {
        list.into_iter().map(|e| Entity {
            entity: e.entity,
            value: e.value,
            start: e.start as u32,
            end: e.end as u32,
            score: e.score as f64,
        }).collect()
    }).collect();
    
    Ok(results)
}
