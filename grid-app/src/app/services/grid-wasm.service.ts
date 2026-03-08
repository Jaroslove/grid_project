import { Injectable, signal, computed } from '@angular/core';
import type {
  RenderFrame,
  HitResult,
  CellRect,
  PivotConfig,
} from '../models/grid.models';

@Injectable({ providedIn: 'root' })
export class GridWasmService {
  private grid: any = null;

  readonly ready = signal(false);
  //readonly cellCount = signal(0);
  readonly groupCount = signal(0);
  //readonly selectedCell = signal<{ row: number; col: number } | null>(null);
  readonly statusMessage = signal('Initializing WASM...');

  async initialize(viewportWidth: number, viewportHeight: number): Promise<void> {
    try {
      // Load WASM from a known path so dev server serves the file (not index.html)
      const wasmUrl = 'assets/wasm/grid_wasm_bg.wasm';
      const wasm = await import('../wasm/grid_wasm.js' as any);
      await wasm.default(wasmUrl);

      this.grid = new wasm.Grid(viewportWidth, viewportHeight);

      // Populate initial data (pivot-style)
      //this.populateInitialPivotData();

      this.ready.set(true);
      this.statusMessage.set('Ready');
      //this.updateStats();
    } catch (err) {
      console.error('WASM init failed:', err);
      this.statusMessage.set(`WASM init failed: ${err}`);
    }
  }

  
  loadData(d: any[]): void { this.grid?.load_data(JSON.stringify(d)); }
  setPivotConfig(c: PivotConfig): void { this.grid?.set_pivot_config(JSON.stringify(c)); }
  buildPivot(): void { this.grid?.build_pivot(); }
  toggleRowCollapse(k: string): void { this.grid?.toggle_row_collapse(k); this.grid?.build_pivot(); }
  toggleColCollapse(k: string): void { this.grid?.toggle_col_collapse(k); this.grid?.build_pivot(); }

  setCell(r: number, c: number, t: string): void { this.grid?.set_cell(r, c, t); }
  getCellText(r: number, c: number): string { return this.grid?.get_cell_text(r, c) ?? ''; }
  clearCell(r: number, c: number): void { this.grid?.clear_cell(r, c); }

  setViewport(w: number, h: number): void { this.grid?.set_viewport(w, h); }
  scrollBy(dx: number, dy: number): void { this.grid?.scroll_by(dx, dy); }
  setScroll(x: number, y: number): void { this.grid?.set_scroll(x, y); }
  getScrollX(): number { return this.grid?.get_scroll_x() ?? 0; }
  getScrollY(): number { return this.grid?.get_scroll_y() ?? 0; }

  select(r: number, c: number): void { this.grid?.select(r, c); }
  selectedCell(): { row: number; col: number } | null {
    const r = this.grid?.sel_row() ?? -1, c = this.grid?.sel_col() ?? -1;
    return r >= 0 && c >= 0 ? { row: r, col: c } : null;
  }
  getSelectedRow(): number { return this.grid?.sel_row() ?? -1; }
  getSelectedCol(): number { return this.grid?.sel_col() ?? -1; }
  moveSelection(dr: number, dc: number): void { this.grid?.move_selection(dr, dc); }

  startEdit(r: number, c: number): void { this.grid?.edit(r, c); }
  stopEdit(): void { this.grid?.edit(-1, -1); }

  startColResize(c: number, x: number): void { this.grid?.start_col_resize(c, x); }
  updateColResize(x: number): void { this.grid?.update_col_resize(x); }
  endColResize(): void { this.grid?.end_col_resize(); }
  isResizing(): boolean { return this.grid?.is_resizing() ?? false; }

  startHDrag(x: number): void { this.grid?.start_h_drag(x); }
  startVDrag(y: number): void { this.grid?.start_v_drag(y); }
  updateHDrag(x: number): void { this.grid?.update_h_drag(x); }
  updateVDrag(y: number): void { this.grid?.update_v_drag(y); }
  endDrag(): void { this.grid?.end_drag(); }
  isDraggingScrollbar(): boolean { return this.grid?.is_dragging_scrollbar() ?? false; }
  clickHTrack(x: number): void { this.grid?.click_h_track(x); }
  clickVTrack(y: number): void { this.grid?.click_v_track(y); }

  renderFrame(): RenderFrame | null {
    if (!this.grid) return null;
    try { return JSON.parse(this.grid.render_frame()); } catch { return null; }
  }
  hitTest(x: number, y: number): HitResult | null {
    if (!this.grid) return null;
    try { return JSON.parse(this.grid.hit_test(x, y)); } catch { return null; }
  }
  cellScreenRect(r: number, c: number): CellRect | null {
    if (!this.grid) return null;
    try { return JSON.parse(this.grid.cell_screen_rect(r, c)); } catch { return null; }
  }
}
