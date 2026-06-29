# Maskara (TypeScript/Node.js SDK)

> High-performance local PII detection, masking, and redaction SDK for TypeScript/JavaScript, powered by a native Rust core.

Maskara delivers enterprise-grade PII detection and redaction with **zero external dependencies, zero weight downloads, and zero runtime setup**. The model weights are quantized and compressed inside the compiled native library binary, running entirely offline and securely in under 25ms.

---

## Features

- **⚡ Zero Setup:** Works instantly out-of-the-box. No Hugging Face downloads, no runtime ONNX setup, and no PyTorch/Python overhead.
- **🦀 Rust Native Engine:** Built on Hugging Face's `candle` ML framework. Performs direct tensor operations on CPU with custom weight-only dynamic quantization.
- **📦 Embedded Weights:** The model weights are embedded inside the library binary, compressed via `zstd`, and loaded into memory on first call in <100ms.
- **🔍 Accurate Offsets:** Character-offset mapping returns exact character index positions of target entities.

---

## Installation

```bash
npm install maskara
```

---

## Usage

```typescript
import { detect, mask, redact } from 'maskara';

const text = "Hi, my phone is +91-9876543210 and my Aadhaar number is 1234-5678-9012.";

// 1. Detect PII entities
const entities = detect(text);
console.log(entities);
/*
[
  { entity: 'PHONE', value: '+91-9876543210', start: 16, end: 30, score: 0.99 },
  { entity: 'AADHAAR', value: '1234-5678-9012', start: 55, end: 71, score: 0.98 }
]
*/

// 2. Mask PII entities (defaults to entity placeholder like [PHONE])
const masked = mask(text);
console.log(masked);
// "Hi, my phone is [PHONE] and my Aadhaar number is [AADHAAR]."

// 3. Custom placeholders or entity filters
const customMask = mask(text, {
  entities: ['AADHAAR'],
  placeholder: 'REDACTED_DATA'
});
console.log(customMask);
// "Hi, my phone is +91-9876543210 and my Aadhaar number is REDACTED_DATA."

// 4. Redact (remove) PII entirely
const redacted = redact(text);
console.log(redacted);
// "Hi, my phone is  and my Aadhaar number is ."
```

---

## Supported Entities

Supports detection of **17 common Indian and international entities**:

- **Indian Identifiers:** `AADHAAR`, `PAN_CARD`
- **Financials:** `CREDIT_CARD`
- **Contact Info:** `PHONE`, `EMAIL`, `ADDRESS`, `IP_ADDRESS`
- **Credentials:** `PASSWORD`, `USERNAME`, `API_KEY`
- **Identity & Documents:** `DRIVER_LICENSE`, `PASSPORT`
- **Personal Details:** `PERSON_NAME`, `DATE_OF_BIRTH`, `GENDER`, `MEDICAL_RECORD`

---

## Build from Source

To compile the native bindings locally, make sure you have the Rust toolchain installed:

```bash
npm install
npm run build
```
This will compile the Rust source, generate the NAPI-RS bindings, and build the TypeScript types under `dist/`.
