/* tslint:disable */
/* eslint-disable */

export class Grid {
    free(): void;
    [Symbol.dispose](): void;
    add_col_group(label: string, members_json: string, parent_id: number): number;
    add_row_group(label: string, members_json: string, parent_id: number): number;
    cell_count(): number;
    cell_screen_rect(row: number, col: number): string;
    clear_cell(row: number, col: number): void;
    edit(r: number, c: number): void;
    edit_col(): number;
    edit_row(): number;
    end_col_resize(): void;
    get_cell_text(row: number, col: number): string;
    get_scroll_x(): number;
    get_scroll_y(): number;
    group_count(): number;
    hit_test(cx: number, cy: number): string;
    is_resizing(): boolean;
    load_cells_json(json: string): void;
    /**
     * Load data in a simple pivot-table shape.
     *
     * Expects JSON like:
     *   [{ "row": "Row A", "col": "Col 1", "value": 10.0 }, ...]
     *
     * It will:
     * - Clear existing cells and groups.
     * - Use row index 0 as header row (column labels).
     * - Use column index 0 as header column (row labels).
     * - Fill numeric cells with the sum of `value` for each (row, col) pair.
     */
    load_pivot_json(json: string): void;
    move_selection(dr: number, dc: number): void;
    constructor(vw: number, vh: number);
    remove_group(group_id: number): void;
    render_frame(): string;
    scroll_by(dx: number, dy: number): void;
    sel_col(): number;
    sel_row(): number;
    select(r: number, c: number): void;
    set_cell(row: number, col: number, text: string): void;
    set_cell_style(row: number, col: number, bg: string, fg: string, bold: boolean): void;
    set_col_width(col: number, w: number): void;
    set_default_col_width(w: number): void;
    set_default_row_height(h: number): void;
    set_row_height(row: number, h: number): void;
    set_scroll(x: number, y: number): void;
    set_viewport(w: number, h: number): void;
    start_col_resize(col: number, start_x: number): void;
    toggle_group(group_id: number): void;
    update_col_resize(current_x: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_grid_free: (a: number, b: number) => void;
    readonly grid_add_col_group: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly grid_add_row_group: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly grid_cell_count: (a: number) => number;
    readonly grid_cell_screen_rect: (a: number, b: number, c: number) => [number, number];
    readonly grid_clear_cell: (a: number, b: number, c: number) => void;
    readonly grid_edit: (a: number, b: number, c: number) => void;
    readonly grid_edit_col: (a: number) => number;
    readonly grid_edit_row: (a: number) => number;
    readonly grid_end_col_resize: (a: number) => void;
    readonly grid_get_cell_text: (a: number, b: number, c: number) => [number, number];
    readonly grid_get_scroll_x: (a: number) => number;
    readonly grid_get_scroll_y: (a: number) => number;
    readonly grid_group_count: (a: number) => number;
    readonly grid_hit_test: (a: number, b: number, c: number) => [number, number];
    readonly grid_is_resizing: (a: number) => number;
    readonly grid_load_cells_json: (a: number, b: number, c: number) => void;
    readonly grid_load_pivot_json: (a: number, b: number, c: number) => void;
    readonly grid_move_selection: (a: number, b: number, c: number) => void;
    readonly grid_new: (a: number, b: number) => number;
    readonly grid_remove_group: (a: number, b: number) => void;
    readonly grid_render_frame: (a: number) => [number, number];
    readonly grid_scroll_by: (a: number, b: number, c: number) => void;
    readonly grid_sel_col: (a: number) => number;
    readonly grid_sel_row: (a: number) => number;
    readonly grid_select: (a: number, b: number, c: number) => void;
    readonly grid_set_cell: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly grid_set_cell_style: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly grid_set_col_width: (a: number, b: number, c: number) => void;
    readonly grid_set_default_col_width: (a: number, b: number) => void;
    readonly grid_set_default_row_height: (a: number, b: number) => void;
    readonly grid_set_row_height: (a: number, b: number, c: number) => void;
    readonly grid_set_scroll: (a: number, b: number, c: number) => void;
    readonly grid_set_viewport: (a: number, b: number, c: number) => void;
    readonly grid_start_col_resize: (a: number, b: number, c: number) => void;
    readonly grid_toggle_group: (a: number, b: number) => void;
    readonly grid_update_col_resize: (a: number, b: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
