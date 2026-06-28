import * as native from '../index';
import { Entity, MaskOptions, RedactOptions } from './types';

export { Entity, MaskOptions, RedactOptions };

export function detect(text: string): Entity[] {
  return native.detect(text);
}

export function mask(text: string, options?: MaskOptions): string {
  return native.mask(text, options);
}

export function redact(text: string, options?: RedactOptions): string {
  return native.redact(text, options);
}

export function detectBatch(texts: string[]): Entity[][] {
  return native.detectBatch(texts);
}
