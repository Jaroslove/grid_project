declare module '*.wasm' {
    const value: any;
    export default value;
  }
  
// Declare the wasm-bindgen generated module
declare module '*grid_wasm' {
    export default function init(input?: RequestInfo | URL): Promise<any>;
    export class Grid {
        constructor(vw: number, vh: number);
        set_cell(row: number, col: number, text: string): void;
        set_cell_style(row: number, col: number, bg: string, fg: string, bold: boolean): void;
        get_cell_text(row: number, col: number): string;
        clear_cell(row: number, col: number): void;
        load_cells_json(json: string): void;
        set_col_width(col: number, w: number): void;
        set_row_height(row: number, h: number): void;
        set_default_col_width(w: number): void;
        set_default_row_height(h: number): void;
        set_viewport(w: number, h: number): void;
        scroll_by(dx: number, dy: number): void;
        set_scroll(x: number, y: number): void;
        get_scroll_x(): number;
        get_scroll_y(): number;
        select(row: number, col: number): void;
        sel_row(): number;
        sel_col(): number;
        edit(row: number, col: number): void;
        edit_row(): number;
        edit_col(): number;
        move_selection(dr: number, dc: number): void;
        start_col_resize(col: number, start_x: number): void;
        update_col_resize(current_x: number): void;
        end_col_resize(): void;
        is_resizing(): boolean;
        add_row_group(label: string, members_json: string, parent_id: number): number;
        add_col_group(label: string, members_json: string, parent_id: number): number;
        toggle_group(group_id: number): void;
        remove_group(group_id: number): void;
        hit_test(cx: number, cy: number): string;
        cell_screen_rect(row: number, col: number): string;
        render_frame(): string;
        cell_count(): number;
        group_count(): number;
        free(): void;
    }
}
