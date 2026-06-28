use std::io::Cursor;
use candle_core::{Device, Result, Tensor};
use candle_nn::{Linear, Module, VarBuilder};
use candle_transformers::models::bert::{BertModel, Config};

pub struct MaskaraEngine {
    model: BertModel,
    classifier: Linear,
    device: Device,
}

impl MaskaraEngine {
    pub fn new() -> Result<Self> {
        let device = Device::Cpu;

        // 1. Decompress embedded weights
        let compressed_weights = include_bytes!("../assets/maskara_int8.zst");
        let decompressed_weights = zstd::stream::decode_all(Cursor::new(compressed_weights))
            .map_err(|e| candle_core::Error::wrap(e))?;

        // 2. Load Safetensors from buffer
        let raw_tensors = candle_core::safetensors::load_buffer(&decompressed_weights, &device)?;
        let mut tensors = std::collections::HashMap::new();

        for (name, tensor) in raw_tensors.iter() {
            if name.ends_with("_scale") {
                continue;
            }
            
            let scale_name = format!("{}_scale", name);
            if let Some(scale_tensor) = raw_tensors.get(&scale_name) {
                // Quantized tensor! Dequantize it from U8 (shifted by 128) to F32 using the scale.
                let scale = scale_tensor.to_scalar::<f32>()? as f64;
                let f32_tensor = tensor.to_dtype(candle_core::DType::F32)?;
                let shifted = f32_tensor.affine(1.0, -128.0)?;
                let dequantized = shifted.affine(scale, 0.0)?;
                tensors.insert(name.clone(), dequantized);
            } else {
                // Non-quantized tensor. Just clone and insert.
                tensors.insert(name.clone(), tensor.clone());
            }
        }

        // 3. Create VarBuilder from the dequantized tensors
        let vb = VarBuilder::from_tensors(tensors, candle_core::DType::F32, &device);

        // 3. Load config
        let config_str = include_str!("../assets/config.json");
        let config: Config = serde_json::from_str(config_str)
            .map_err(|e| candle_core::Error::wrap(e))?;

        // 4. Instantiate the model
        let model = BertModel::load(vb.pp("bert"), &config)?;
        
        // bert-base-uncased has hidden size 768. Number of labels in Phase 2 is 35 (17 entities * 2 + 1).
        let classifier = candle_nn::linear(config.hidden_size, 35, vb.pp("classifier"))?;

        Ok(Self {
            model,
            classifier,
            device,
        })
    }

    pub fn predict(&self, input_ids: &[u32], token_type_ids: &[u32]) -> Result<Tensor> {
        let len = input_ids.len();
        let input_ids_t = Tensor::from_slice(input_ids, (1, len), &self.device)?;
        let token_type_ids_t = Tensor::from_slice(token_type_ids, (1, len), &self.device)?;

        // Forward pass through BERT base
        let sequence_output = self.model.forward(&input_ids_t, &token_type_ids_t, None)?;
        
        // Sequence classification head
        let logits = self.classifier.forward(&sequence_output)?;
        
        // Remove batch dimension -> [seq_len, 35]
        let logits = logits.squeeze(0)?;
        
        // Softmax over label dimension (index 1)
        let probs = candle_nn::ops::softmax(&logits, 1)?;
        Ok(probs)
    }
}
