// TypeScript types for SWF Editor

export interface SWFHeader {
  signature: string;
  version: number;
  file_length: number;
  frame_size: Rect;
  frame_rate: number;
  frame_count: number;
  compressed: boolean;
}

export interface Rect {
  x_min: number;
  x_max: number;
  y_min: number;
  y_max: number;
}

export interface SWFFile {
  path: string;
  header: SWFHeader;
  tags: Tag[];
  resources: Resources;
}

export interface Tag {
  type: string;
  [key: string]: any;
}

export interface Resources {
  images: Record<number, ImageResource>;
  sounds: Record<number, SoundResource>;
  sprites: Record<number, SpriteResource>;
  scripts: Record<number, ScriptResource>;
  texts: Record<number, TextResource>;
  fonts: Record<number, FontResource>;
  shapes: Record<number, ShapeResource>;
  binary_data: Record<number, BinaryDataResource>;
}

export interface ImageResource {
  id: number;
  width: number;
  height: number;
  format: string;
  data: number[];
}

export interface SoundResource {
  id: number;
  format: string;
  sample_rate: number;
  stereo: boolean;
  sample_count: number;
  data: number[];
}

export interface SpriteResource {
  id: number;
  frame_count: number;
  tags: Tag[];
}

export interface ScriptResource {
  id: number;
  name: string;
  bytecode: number[];
  decompiled?: string;
  script_type: string;
}

export interface TextResource {
  id: number;
  bounds: Rect;
  text: string;
  raw_data: number[];
}

export interface FontResource {
  id: number;
  name?: string;
  num_glyphs: number;
  data: number[];
}

export interface ShapeResource {
  id: number;
  bounds: Rect;
  edge_bounds?: Rect;
  data: number[];
}

export interface BinaryDataResource {
  id: number;
  data: number[];
}

export interface ResourceInfo {
  id: number;
  resource_type: string;
  name?: string;
  size: number;
  metadata?: Record<string, string>;
}

export type ResourceType = "images" | "sounds" | "sprites" | "scripts" | "texts" | "fonts" | "shapes" | "binary_data";
