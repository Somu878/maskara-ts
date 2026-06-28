use crate::api::Entity;

pub const SUPPORTED_ENTITIES: &[&str] = &[
    "ADDRESS", "API_KEY", "CREDIT_CARD", "DATE_OF_BIRTH", "DRIVER_LICENSE",
    "EMAIL", "IP_ADDRESS", "PASSWORD", "PERSON_NAME", "PHONE", "SSN",
    "USERNAME", "AADHAAR", "PAN_CARD", "PASSPORT", "UPI_ID", "VEHICLE_REG",
];

pub fn get_label_name(id: usize) -> &'static str {
    if id == 0 {
        return "O";
    }
    if id <= 17 {
        return SUPPORTED_ENTITIES[id - 1];
    }
    if id <= 34 {
        return SUPPORTED_ENTITIES[id - 18];
    }
    "O"
}

pub fn is_b_tag(id: usize) -> bool {
    id >= 1 && id <= 17
}

pub fn is_i_tag(id: usize) -> bool {
    id >= 18 && id <= 34
}

#[derive(Debug, Clone)]
pub struct TokenPrediction {
    pub label_id: usize,
    pub score: f32,
    pub start: usize,
    pub end: usize,
}

pub fn decode_bio_spans(predictions: &[TokenPrediction], text: &str) -> Vec<Entity> {
    let mut spans = Vec::new();
    let mut current_entity: Option<String> = None;
    let mut current_start = 0;
    let mut current_end = 0;
    let mut current_scores = Vec::new();

    for pred in predictions {
        if pred.start == 0 && pred.end == 0 {
            continue; // Skip special/empty tokens
        }

        let label_id = pred.label_id;
        if is_b_tag(label_id) {
            // Close current entity if any
            if let Some(ref ent) = current_entity {
                let avg_score = if current_scores.is_empty() { 0.0 } else {
                    current_scores.iter().sum::<f32>() / current_scores.len() as f32
                };
                let value = text.get(current_start..current_end).unwrap_or("").to_string();
                spans.push(Entity {
                    entity: ent.clone(),
                    value,
                    start: current_start,
                    end: current_end,
                    score: avg_score,
                });
            }

            let ent_name = get_label_name(label_id).to_string();
            current_entity = Some(ent_name);
            current_start = pred.start;
            current_end = pred.end;
            current_scores = vec![pred.score];
        } else if is_i_tag(label_id) {
            let ent_name = get_label_name(label_id);
            if let Some(ref current_ent) = current_entity {
                if current_ent == ent_name {
                    current_end = pred.end;
                    current_scores.push(pred.score);
                } else {
                    // Mismatched I tag, close current and reset
                    let avg_score = if current_scores.is_empty() { 0.0 } else {
                        current_scores.iter().sum::<f32>() / current_scores.len() as f32
                    };
                    let value = text.get(current_start..current_end).unwrap_or("").to_string();
                    spans.push(Entity {
                        entity: current_ent.clone(),
                        value,
                        start: current_start,
                        end: current_end,
                        score: avg_score,
                    });
                    current_entity = None;
                    current_scores.clear();
                }
            }
        } else {
            // O tag: Close current entity
            if let Some(ref ent) = current_entity {
                let avg_score = if current_scores.is_empty() { 0.0 } else {
                    current_scores.iter().sum::<f32>() / current_scores.len() as f32
                };
                let value = text.get(current_start..current_end).unwrap_or("").to_string();
                spans.push(Entity {
                    entity: ent.clone(),
                    value,
                    start: current_start,
                    end: current_end,
                    score: avg_score,
                });
                current_entity = None;
                current_scores.clear();
            }
        }
    }

    // Flush any remaining active entity
    if let Some(ref ent) = current_entity {
        let avg_score = if current_scores.is_empty() { 0.0 } else {
            current_scores.iter().sum::<f32>() / current_scores.len() as f32
        };
        let value = text.get(current_start..current_end).unwrap_or("").to_string();
        spans.push(Entity {
            entity: ent.clone(),
            value,
            start: current_start,
            end: current_end,
            score: avg_score,
        });
    }

    spans
}
