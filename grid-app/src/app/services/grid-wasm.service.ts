import { Injectable, signal, computed } from '@angular/core';
import type { RenderFrame, HitResult, CellRect } from '../models/grid.models';

@Injectable({ providedIn: 'root' })
export class GridWasmService {
  private grid: any = null;

  readonly ready = signal(false);
  readonly cellCount = signal(0);
  readonly groupCount = signal(0);
  readonly selectedCell = signal<{ row: number; col: number } | null>(null);
  readonly statusMessage = signal('Initializing WASM...');

  async initialize(viewportWidth: number, viewportHeight: number): Promise<void> {
    try {
      // Load WASM from a known path so dev server serves the file (not index.html)
      const wasmUrl = 'assets/wasm/grid_wasm_bg.wasm';
      const wasm = await import('../wasm/grid_wasm.js' as any);
      await wasm.default(wasmUrl);

      this.grid = new wasm.Grid(viewportWidth, viewportHeight);

      // Populate initial data (pivot-style)
      this.populateInitialPivotData();

      this.ready.set(true);
      this.statusMessage.set('Ready');
      this.updateStats();
    } catch (err) {
      console.error('WASM init failed:', err);
      this.statusMessage.set(`WASM init failed: ${err}`);
    }
  }

  // Simple demo pivot data: Region x Year with Sales totals.
  private populateInitialPivotData(): void {
    if (!this.grid) return;

    const records = [
      { row: 'North', col: '2023', value: 120 },
      { row: 'North', col: '2024', value: 180 },
      { row: 'South', col: '2023', value: 90 },
      { row: 'South', col: '2024', value: 150 },
      { row: 'East',  col: '2023', value: 60 },
      { row: 'East',  col: '2024', value: 110 },
      { row: 'West',  col: '2023', value: 75 },
      { row: 'West',  col: '2024', value: 95 },
    ];

    this.grid.load_pivot_json(JSON.stringify(records));
  }

  // Public API to load arbitrary pivot data from the Angular side.
  loadPivotData(records: { row: string; col: string; value: number }[]): void {
    if (!this.grid) return;
    this.grid.load_pivot_json(JSON.stringify(records));
    this.updateStats();
  }

  // ─── Grid operations ────────────────────────────────────────

  setViewport(w: number, h: number): void {
    this.grid?.set_viewport(w, h);
  }

  scrollBy(dx: number, dy: number): void {
    this.grid?.scroll_by(dx, dy);
  }

  setScroll(x: number, y: number): void {
    this.grid?.set_scroll(x, y);
  }

  getScrollX(): number { return this.grid?.get_scroll_x() ?? 0; }
  getScrollY(): number { return this.grid?.get_scroll_y() ?? 0; }

  // ─── Selection ──────────────────────────────────────────────

  select(row: number, col: number): void {
    this.grid?.select(row, col);
    this.selectedCell.set(row >= 0 && col >= 0 ? { row, col } : null);
  }

  moveSelection(dr: number, dc: number): void {
    this.grid?.move_selection(dr, dc);
    const r = this.grid?.sel_row() ?? -1;
    const c = this.grid?.sel_col() ?? -1;
    this.selectedCell.set(r >= 0 && c >= 0 ? { row: r, col: c } : null);
  }

  getSelectedRow(): number { return this.grid?.sel_row() ?? -1; }
  getSelectedCol(): number { return this.grid?.sel_col() ?? -1; }

  // ─── Editing ────────────────────────────────────────────────

  startEdit(row: number, col: number): void {
    this.grid?.edit(row, col);
  }

  stopEdit(): void {
    this.grid?.edit(-1, -1);
  }

  getCellText(row: number, col: number): string {
    return this.grid?.get_cell_text(row, col) ?? '';
  }

  setCell(row: number, col: number, text: string): void {
    this.grid?.set_cell(row, col, text);
    this.updateStats();
  }

  clearCell(row: number, col: number): void {
    this.grid?.clear_cell(row, col);
    this.updateStats();
  }

  cellScreenRect(row: number, col: number): CellRect | null {
    if (!this.grid) return null;
    try {
      return JSON.parse(this.grid.cell_screen_rect(row, col));
    } catch { return null; }
  }

  // ─── Column resize ─────────────────────────────────────────

  startColResize(col: number, startX: number): void {
    this.grid?.start_col_resize(col, startX);
  }

  updateColResize(currentX: number): void {
    this.grid?.update_col_resize(currentX);
  }

  endColResize(): void {
    this.grid?.end_col_resize();
  }

  isResizing(): boolean {
    return this.grid?.is_resizing() ?? false;
  }

  // ─── Groups ─────────────────────────────────────────────────

  addRowGroup(label: string, members: number[], parentId: number = -1): number {
    const id = this.grid?.add_row_group(label, JSON.stringify(members), parentId) ?? 0;
    this.updateStats();
    return id;
  }

  addColGroup(label: string, members: number[], parentId: number = -1): number {
    const id = this.grid?.add_col_group(label, JSON.stringify(members), parentId) ?? 0;
    this.updateStats();
    return id;
  }

  toggleGroup(groupId: number): void {
    this.grid?.toggle_group(groupId);
  }

  removeGroup(groupId: number): void {
    this.grid?.remove_group(groupId);
    this.updateStats();
  }

  // ─── Hit test ───────────────────────────────────────────────

  hitTest(cx: number, cy: number): HitResult | null {
    if (!this.grid) return null;
    try {
      return JSON.parse(this.grid.hit_test(cx, cy));
    } catch { return null; }
  }

  // ─── Render ─────────────────────────────────────────────────

  renderFrame(): RenderFrame | null {
    if (!this.grid) return null;
    try {
      return JSON.parse(this.grid.render_frame());
    } catch { return null; }
  }

  // ─── Bulk data ──────────────────────────────────────────────

  loadBulkData(count: number): number {
    if (!this.grid) return 0;
    const t0 = performance.now();
    const data: [number, number, string][] = [];
    const rows = Math.ceil(Math.sqrt(count));
    const cols = Math.ceil(count / rows);
    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        data.push([r, c, `${r + 1}:${c + 1}`]);
      }
    }
    this.grid.load_cells_json(JSON.stringify(data));
    this.updateStats();
    return performance.now() - t0;
  }

  // ─── Stats ──────────────────────────────────────────────────

  private updateStats(): void {
    this.cellCount.set(this.grid?.cell_count() ?? 0);
    this.groupCount.set(this.grid?.group_count() ?? 0);
  }
}
