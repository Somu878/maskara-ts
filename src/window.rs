use crate::api::Entity;

/// Merge overlapping entities by keeping the one with the highest confidence score (greedy NMS).
pub fn merge_overlapping_entities(mut entities: Vec<Entity>) -> Vec<Entity> {
    // Sort by score descending
    entities.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let mut merged: Vec<Entity> = Vec::new();

    for ent in entities {
        // Check if this entity overlaps with any already selected entity
        let mut overlaps = false;
        for existing in &merged {
            // Check character index overlap: max(start1, start2) < min(end1, end2)
            let max_start = std::cmp::max(ent.start, existing.start);
            let min_end = std::cmp::min(ent.end, existing.end);
            if max_start < min_end {
                overlaps = true;
                break;
            }
        }

        if !overlaps {
            merged.push(ent);
        }
    }

    // Sort the final merged entities by start position for deterministic output
    merged.sort_by_key(|e| e.start);
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_no_overlap() {
        let entities = vec![
            Entity { entity: "PERSON_NAME".into(), value: "A".into(), start: 0, end: 5, score: 0.9 },
            Entity { entity: "PHONE".into(), value: "B".into(), start: 10, end: 15, score: 0.8 },
        ];
        let merged = merge_overlapping_entities(entities);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].start, 0);
        assert_eq!(merged[1].start, 10);
    }

    #[test]
    fn test_merge_with_overlap() {
        let entities = vec![
            Entity { entity: "PERSON_NAME".into(), value: "Alice".into(), start: 0, end: 10, score: 0.95 },
            Entity { entity: "PERSON_NAME".into(), value: "Alice Smith".into(), start: 0, end: 15, score: 0.99 },
            Entity { entity: "ADDRESS".into(), value: "Smith".into(), start: 10, end: 15, score: 0.6 },
        ];
        let merged = merge_overlapping_entities(entities);
        // The highest confidence is "Alice Smith" (0.99) at 0..15.
        // It overlaps with "Alice" (0..10) and "Smith" (10..15), so both are discarded.
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].value, "Alice Smith");
        assert_eq!(merged[0].score, 0.99);
    }
}
