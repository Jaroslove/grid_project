/* tslint:disable */
/* eslint-disable */

export class Grid {
    free(): void;
    [Symbol.dispose](): void;
    build_pivot(): void;
    cell_count(): number;
    cell_screen_rect(r: number, c: number): string;
    clear_cell(r: number, c: number): void;
    click_h_track(x: number): void;
    click_v_track(y: number): void;
    edit(r: number, c: number): void;
    edit_col(): number;
    edit_row(): number;
    end_col_resize(): void;
    end_drag(): void;
    get_cell_text(r: number, c: number): string;
    get_scroll_x(): number;
    get_scroll_y(): number;
    get_total_cols(): number;
    get_total_rows(): number;
    hit_test(cx: number, cy: number): string;
    is_dragging_scrollbar(): boolean;
    is_resizing(): boolean;
    load_data(json: string): void;
    move_selection(dr: number, dc: number): void;
    constructor(w: number, h: number);
    render_frame(): string;
    scroll_by(dx: number, dy: number): void;
    sel_col(): number;
    sel_row(): number;
    select(r: number, c: number): void;
    set_cell(r: number, c: number, t: string): void;
    set_col_width(c: number, w: number): void;
    set_pivot_config(json: string): void;
    set_scroll(x: number, y: number): void;
    set_viewport(w: number, h: number): void;
    start_col_resize(c: number, x: number): void;
    start_h_drag(x: number): void;
    start_v_drag(y: number): void;
    toggle_col_collapse(k: string): void;
    toggle_row_collapse(k: string): void;
    update_col_resize(x: number): void;
    update_h_drag(x: number): void;
    update_v_drag(y: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_grid_free: (a: number, b: number) => void;
    readonly grid_build_pivot: (a: number) => void;
    readonly grid_cell_count: (a: number) => number;
    readonly grid_cell_screen_rect: (a: number, b: number, c: number) => [number, number];
    readonly grid_clear_cell: (a: number, b: number, c: number) => void;
    readonly grid_click_h_track: (a: number, b: number) => void;
    readonly grid_click_v_track: (a: number, b: number) => void;
    readonly grid_edit: (a: number, b: number, c: number) => void;
    readonly grid_edit_col: (a: number) => number;
    readonly grid_edit_row: (a: number) => number;
    readonly grid_end_col_resize: (a: number) => void;
    readonly grid_end_drag: (a: number) => void;
    readonly grid_get_cell_text: (a: number, b: number, c: number) => [number, number];
    readonly grid_get_scroll_x: (a: number) => number;
    readonly grid_get_scroll_y: (a: number) => number;
    readonly grid_get_total_cols: (a: number) => number;
    readonly grid_get_total_rows: (a: number) => number;
    readonly grid_hit_test: (a: number, b: number, c: number) => [number, number];
    readonly grid_is_dragging_scrollbar: (a: number) => number;
    readonly grid_is_resizing: (a: number) => number;
    readonly grid_load_data: (a: number, b: number, c: number) => void;
    readonly grid_move_selection: (a: number, b: number, c: number) => void;
    readonly grid_new: (a: number, b: number) => number;
    readonly grid_render_frame: (a: number) => [number, number];
    readonly grid_scroll_by: (a: number, b: number, c: number) => void;
    readonly grid_sel_col: (a: number) => number;
    readonly grid_sel_row: (a: number) => number;
    readonly grid_select: (a: number, b: number, c: number) => void;
    readonly grid_set_cell: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly grid_set_col_width: (a: number, b: number, c: number) => void;
    readonly grid_set_pivot_config: (a: number, b: number, c: number) => void;
    readonly grid_set_scroll: (a: number, b: number, c: number) => void;
    readonly grid_set_viewport: (a: number, b: number, c: number) => void;
    readonly grid_start_col_resize: (a: number, b: number, c: number) => void;
    readonly grid_start_h_drag: (a: number, b: number) => void;
    readonly grid_start_v_drag: (a: number, b: number) => void;
    readonly grid_toggle_col_collapse: (a: number, b: number, c: number) => void;
    readonly grid_toggle_row_collapse: (a: number, b: number, c: number) => void;
    readonly grid_update_col_resize: (a: number, b: number) => void;
    readonly grid_update_h_drag: (a: number, b: number) => void;
    readonly grid_update_v_drag: (a: number, b: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
