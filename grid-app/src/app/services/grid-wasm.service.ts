import { Injectable, signal, computed } from '@angular/core';
import type { Frame, HitResult, CellRect, PivotConfig } from '../models/grid.models';

@Injectable({ providedIn: 'root' })
export class GridWasmService {
  private g: any = null;

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

      this.g = new wasm.Grid(viewportWidth, viewportHeight);

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

  
  
  loadData(d: any[]): void { this.g?.load_data(JSON.stringify(d)); }
  setPivotConfig(c: PivotConfig): void { this.g?.set_pivot_config(JSON.stringify(c)); }
  buildPivot(): void { this.g?.build_pivot(); }

  toggleRowCollapse(k: string): void {
    this.g?.toggle_row_collapse(k);
    this.g?.build_pivot();
  }
  toggleColCollapse(k: string): void {
    this.g?.toggle_col_collapse(k);
    this.g?.build_pivot();
  }

  setCell(r: number, c: number, t: string): void { this.g?.set_cell(r, c, t); }
  getCellText(r: number, c: number): string { return this.g?.get_cell_text(r, c) ?? ''; }
  clearCell(r: number, c: number): void { this.g?.clear_cell(r, c); }

  setViewport(w: number, h: number): void { this.g?.set_viewport(w, h); }
  scrollBy(dx: number, dy: number): void { this.g?.scroll_by(dx, dy); }
  setScroll(x: number, y: number): void { this.g?.set_scroll(x, y); }
  getScrollX(): number { return this.g?.get_scroll_x() ?? 0; }
  getScrollY(): number { return this.g?.get_scroll_y() ?? 0; }

  select(r: number, c: number): void { this.g?.select(r, c); }
  selectedCell(): { row: number; col: number } | null {
    const r = this.g?.sel_row() ?? -1, c = this.g?.sel_col() ?? -1;
    return r >= 0 && c >= 0 ? { row: r, col: c } : null;
  }
  getSelectedRow(): number { return this.g?.sel_row() ?? -1; }
  getSelectedCol(): number { return this.g?.sel_col() ?? -1; }
  moveSelection(dr: number, dc: number): void { this.g?.move_selection(dr, dc); }

  startEdit(r: number, c: number): void { this.g?.edit(r, c); }
  stopEdit(): void { this.g?.edit(-1, -1); }

  startColResize(c: number, x: number): void { this.g?.start_col_resize(c, x); }
  updateColResize(x: number): void { this.g?.update_col_resize(x); }
  endColResize(): void { this.g?.end_col_resize(); }
  isResizing(): boolean { return this.g?.is_resizing() ?? false; }

  startHDrag(x: number): void { this.g?.start_h_drag(x); }
  startVDrag(y: number): void { this.g?.start_v_drag(y); }
  updateHDrag(x: number): void { this.g?.update_h_drag(x); }
  updateVDrag(y: number): void { this.g?.update_v_drag(y); }
  endDrag(): void { this.g?.end_drag(); }
  isDraggingScrollbar(): boolean { return this.g?.is_dragging_scrollbar() ?? false; }
  clickHTrack(x: number): void { this.g?.click_h_track(x); }
  clickVTrack(y: number): void { this.g?.click_v_track(y); }

  renderFrame(): Frame | null {
    if (!this.g) return null;
    try { return JSON.parse(this.g.render_frame()); } catch { return null; }
  }
  hitTest(x: number, y: number): HitResult | null {
    if (!this.g) return null;
    try { return JSON.parse(this.g.hit_test(x, y)); } catch { return null; }
  }
  cellScreenRect(r: number, c: number): CellRect | null {
    if (!this.g) return null;
    try { return JSON.parse(this.g.cell_screen_rect(r, c)); } catch { return null; }
  }
}
