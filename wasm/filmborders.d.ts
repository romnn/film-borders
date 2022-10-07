declare namespace wasm_bindgen {
	/* tslint:disable */
	/* eslint-disable */
	/**
	*/
	export enum FitMode {
	  Image,
	  Border,
	}
	/**
	*/
	export enum Builtin {
	  Border120_1,
	}
	/**
	*/
	export enum Rotation {
	  Rotate0,
	  Rotate90,
	  Rotate180,
	  Rotate270,
	}
	/**
	*/
	export class Border {
	  free(): void;
	/**
	* @param {ImageData | undefined} custom
	* @param {number | undefined} builtin
	*/
	  constructor(custom?: ImageData, builtin?: number);
	/**
	* @param {ImageData} data
	* @returns {Border}
	*/
	  static from_image_data(data: ImageData): Border;
	/**
	* @param {number} builtin
	* @returns {Border}
	*/
	  static builtin(builtin: number): Border;
	}
	/**
	*/
	export class BoundedSize {
	  free(): void;
	/**
	*/
	  constructor();
	/**
	*/
	  height?: number;
	/**
	*/
	  width?: number;
	}
	/**
	*/
	export class Color {
	  free(): void;
	/**
	* @param {string} hex
	*/
	  constructor(hex: string);
	/**
	* @param {number} r
	* @param {number} g
	* @param {number} b
	* @returns {Color}
	*/
	  static rgb(r: number, g: number, b: number): Color;
	/**
	* @param {number} r
	* @param {number} g
	* @param {number} b
	* @param {number} a
	* @returns {Color}
	*/
	  static rgba(r: number, g: number, b: number, a: number): Color;
	/**
	* @returns {Color}
	*/
	  static clear(): Color;
	/**
	* @returns {Color}
	*/
	  static black(): Color;
	/**
	* @returns {Color}
	*/
	  static white(): Color;
	/**
	* @returns {Color}
	*/
	  static gray(): Color;
	}
	/**
	*/
	export class Image {
	  free(): void;
	/**
	* @param {HTMLCanvasElement} canvas
	* @param {CanvasRenderingContext2D} ctx
	* @returns {Image}
	*/
	  static from_canvas(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D): Image;
	/**
	* @param {ImageData} data
	* @returns {Image}
	*/
	  static from_image_data(data: ImageData): Image;
	}
	/**
	*/
	export class ImageBorders {
	  free(): void;
	/**
	* @param {HTMLCanvasElement} canvas
	* @param {CanvasRenderingContext2D} ctx
	* @returns {ImageBorders}
	*/
	  static from_canvas(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D): ImageBorders;
	/**
	* @param {ImageData} data
	* @returns {ImageBorders}
	*/
	  static from_image_data(data: ImageData): ImageBorders;
	/**
	* @param {HTMLCanvasElement} canvas
	* @param {CanvasRenderingContext2D} ctx
	* @returns {ImageData}
	*/
	  static to_image_data(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D): ImageData;
	/**
	* @param {Border} border
	* @param {Options} options
	* @returns {ImageData}
	*/
	  render(border: Border, options: Options): ImageData;
	}
	/**
	*/
	export class Options {
	  free(): void;
	/**
	*/
	  constructor();
	/**
	* @param {string} val
	* @returns {Options}
	*/
	  static deserialize(val: string): Options;
	/**
	* @returns {string}
	*/
	  serialize(): string;
	/**
	*/
	  background_color?: Color;
	/**
	*/
	  border_rotation: number;
	/**
	*/
	  crop?: Sides;
	/**
	*/
	  frame_color: Color;
	/**
	*/
	  frame_width: Sides;
	/**
	*/
	  image_rotation: number;
	/**
	*/
	  margin: number;
	/**
	*/
	  mode: number;
	/**
	*/
	  output_size: BoundedSize;
	/**
	*/
	  output_size_bounds: BoundedSize;
	/**
	*/
	  preview: boolean;
	/**
	*/
	  scale_factor: number;
	}
	/**
	*/
	export class Point {
	  free(): void;
	/**
	*/
	  constructor();
	/**
	* @returns {Point}
	*/
	  static origin(): Point;
	/**
	*/
	  x: bigint;
	/**
	*/
	  y: bigint;
	}
	/**
	*/
	export class Rect {
	  free(): void;
	/**
	*/
	  bottom: bigint;
	/**
	*/
	  left: bigint;
	/**
	*/
	  right: bigint;
	/**
	*/
	  top: bigint;
	}
	/**
	*/
	export class ResultSize {
	  free(): void;
	}
	/**
	*/
	export class Sides {
	  free(): void;
	/**
	*/
	  constructor();
	/**
	* @param {number} side
	* @returns {Sides}
	*/
	  static uniform(side: number): Sides;
	/**
	*/
	  bottom: number;
	/**
	*/
	  left: number;
	/**
	*/
	  right: number;
	/**
	*/
	  top: number;
	}
	/**
	*/
	export class Size {
	  free(): void;
	/**
	*/
	  constructor();
	/**
	*/
	  height: number;
	/**
	*/
	  width: number;
	}
	
}

declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_boundedsize_free: (a: number) => void;
  readonly __wbg_get_boundedsize_width: (a: number, b: number) => void;
  readonly __wbg_set_boundedsize_width: (a: number, b: number, c: number) => void;
  readonly __wbg_get_boundedsize_height: (a: number, b: number) => void;
  readonly __wbg_set_boundedsize_height: (a: number, b: number, c: number) => void;
  readonly boundedsize_new: () => number;
  readonly __wbg_border_free: (a: number) => void;
  readonly border_new: (a: number, b: number) => number;
  readonly border_from_image_data: (a: number) => number;
  readonly border_builtin: (a: number) => number;
  readonly __wbg_image_free: (a: number) => void;
  readonly image_from_canvas: (a: number, b: number, c: number) => void;
  readonly image_from_image_data: (a: number, b: number) => void;
  readonly __wbg_imageborders_free: (a: number) => void;
  readonly imageborders_from_canvas: (a: number, b: number, c: number) => void;
  readonly imageborders_from_image_data: (a: number, b: number) => void;
  readonly imageborders_to_image_data: (a: number, b: number, c: number) => void;
  readonly imageborders_render: (a: number, b: number, c: number, d: number) => void;
  readonly __wbg_color_free: (a: number) => void;
  readonly color_hex: (a: number, b: number, c: number) => void;
  readonly color_rgb: (a: number, b: number, c: number) => number;
  readonly color_rgba: (a: number, b: number, c: number, d: number) => number;
  readonly color_clear: () => number;
  readonly color_black: () => number;
  readonly color_white: () => number;
  readonly color_gray: () => number;
  readonly __wbg_point_free: (a: number) => void;
  readonly __wbg_get_point_x: (a: number, b: number) => void;
  readonly __wbg_set_point_x: (a: number, b: number, c: number) => void;
  readonly __wbg_get_point_y: (a: number, b: number) => void;
  readonly __wbg_set_point_y: (a: number, b: number, c: number) => void;
  readonly point_new: () => number;
  readonly point_origin: () => number;
  readonly __wbg_options_free: (a: number) => void;
  readonly __wbg_get_options_output_size: (a: number) => number;
  readonly __wbg_set_options_output_size: (a: number, b: number) => void;
  readonly __wbg_get_options_output_size_bounds: (a: number) => number;
  readonly __wbg_set_options_output_size_bounds: (a: number, b: number) => void;
  readonly __wbg_get_options_scale_factor: (a: number) => number;
  readonly __wbg_set_options_scale_factor: (a: number, b: number) => void;
  readonly __wbg_get_options_margin: (a: number) => number;
  readonly __wbg_set_options_margin: (a: number, b: number) => void;
  readonly __wbg_get_options_mode: (a: number) => number;
  readonly __wbg_set_options_mode: (a: number, b: number) => void;
  readonly __wbg_get_options_crop: (a: number) => number;
  readonly __wbg_set_options_crop: (a: number, b: number) => void;
  readonly __wbg_get_options_frame_width: (a: number) => number;
  readonly __wbg_set_options_frame_width: (a: number, b: number) => void;
  readonly __wbg_get_options_image_rotation: (a: number) => number;
  readonly __wbg_set_options_image_rotation: (a: number, b: number) => void;
  readonly __wbg_get_options_border_rotation: (a: number) => number;
  readonly __wbg_set_options_border_rotation: (a: number, b: number) => void;
  readonly __wbg_get_options_frame_color: (a: number) => number;
  readonly __wbg_set_options_frame_color: (a: number, b: number) => void;
  readonly __wbg_get_options_background_color: (a: number) => number;
  readonly __wbg_set_options_background_color: (a: number, b: number) => void;
  readonly __wbg_get_options_preview: (a: number) => number;
  readonly __wbg_set_options_preview: (a: number, b: number) => void;
  readonly options_new: () => number;
  readonly options_deserialize: (a: number, b: number, c: number) => void;
  readonly options_serialize: (a: number, b: number) => void;
  readonly __wbg_rect_free: (a: number) => void;
  readonly __wbg_get_rect_top: (a: number, b: number) => void;
  readonly __wbg_set_rect_top: (a: number, b: number, c: number) => void;
  readonly __wbg_get_rect_left: (a: number, b: number) => void;
  readonly __wbg_set_rect_left: (a: number, b: number, c: number) => void;
  readonly __wbg_get_rect_bottom: (a: number, b: number) => void;
  readonly __wbg_set_rect_bottom: (a: number, b: number, c: number) => void;
  readonly __wbg_get_rect_right: (a: number, b: number) => void;
  readonly __wbg_set_rect_right: (a: number, b: number, c: number) => void;
  readonly __wbg_size_free: (a: number) => void;
  readonly __wbg_get_size_width: (a: number) => number;
  readonly __wbg_set_size_width: (a: number, b: number) => void;
  readonly __wbg_get_size_height: (a: number) => number;
  readonly __wbg_set_size_height: (a: number, b: number) => void;
  readonly size_new: () => number;
  readonly __wbg_sides_free: (a: number) => void;
  readonly __wbg_get_sides_top: (a: number) => number;
  readonly __wbg_set_sides_top: (a: number, b: number) => void;
  readonly __wbg_get_sides_left: (a: number) => number;
  readonly __wbg_set_sides_left: (a: number, b: number) => void;
  readonly __wbg_get_sides_bottom: (a: number) => number;
  readonly __wbg_set_sides_bottom: (a: number, b: number) => void;
  readonly __wbg_get_sides_right: (a: number) => number;
  readonly __wbg_set_sides_right: (a: number, b: number) => void;
  readonly sides_new: () => number;
  readonly sides_uniform: (a: number) => number;
  readonly __wbg_resultsize_free: (a: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
declare function wasm_bindgen (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
