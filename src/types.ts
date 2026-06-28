export interface Entity {
  entity: string;
  value: string;
  start: number;
  end: number;
  score: number;
}

export interface MaskOptions {
  entities?: string[];
  threshold?: number;
  placeholder?: string;
}

export interface RedactOptions {
  entities?: string[];
  threshold?: number;
}
