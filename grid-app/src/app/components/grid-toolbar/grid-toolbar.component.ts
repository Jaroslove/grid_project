import { Component, output, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { GridWasmService } from '../../services/grid-wasm.service';

@Component({
  selector: 'app-grid-toolbar',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './grid-toolbar.component.html',
  styleUrl: './grid-toolbar.component.scss',
})
export class GridToolbarComponent {
  private gridService = inject(GridWasmService);

  readonly cellCount = this.gridService.cellCount;
  readonly groupCount = this.gridService.groupCount;
  readonly selectedCell = this.gridService.selectedCell;
  readonly statusMessage = this.gridService.statusMessage;

  readonly requestRender = output<void>();

  private lastRowGroupId: number | null = null;

  populateCells(): void {
    const ms = this.gridService.loadBulkData(10000);
    this.gridService.statusMessage.set(`Loaded 10,000 cells in ${ms.toFixed(1)}ms`);
    this.requestRender.emit();
  }

  addRowGroup(): void {
    const members = [3, 4, 5, 6, 7, 8];
    this.lastRowGroupId = this.gridService.addRowGroup('Rows 4–9', members);
    this.gridService.statusMessage.set(`Row group created (id=${this.lastRowGroupId})`);
    this.requestRender.emit();
  }

  addNestedRowGroup(): void {
    if (!this.lastRowGroupId) {
      this.gridService.statusMessage.set('Create parent row group first');
      return;
    }
    const id = this.gridService.addRowGroup('Nested 5–7', [4, 5, 6], this.lastRowGroupId);
    this.gridService.statusMessage.set(`Nested row group (id=${id}) inside group ${this.lastRowGroupId}`);
    this.requestRender.emit();
  }

  addColGroup(): void {
    const members = [1, 2, 3, 4];
    const id = this.gridService.addColGroup('Cols B–E', members);
    this.gridService.statusMessage.set(`Column group created (id=${id})`);
    this.requestRender.emit();
  }

  populate100k(): void {
    const ms = this.gridService.loadBulkData(100000);
    this.gridService.statusMessage.set(`Loaded 100k cells in ${ms.toFixed(1)}ms`);
    this.requestRender.emit();
  }
}
