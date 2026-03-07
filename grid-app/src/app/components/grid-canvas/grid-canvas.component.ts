import {
    Component,
    ElementRef,
    AfterViewInit,
    OnDestroy,
    ViewChild,
    NgZone,
    inject,
    HostListener,
    ChangeDetectionStrategy,
  } from '@angular/core';
  import { CommonModule } from '@angular/common';
  import { FormsModule } from '@angular/forms';
  import { GridWasmService } from '../../services/grid-wasm.service';
  import { CanvasRenderer } from '../../renderers/canvas-renderer';
  
  @Component({
    selector: 'app-grid-canvas',
    standalone: true,
    imports: [CommonModule, FormsModule],
    templateUrl: './grid-canvas.component.html',
    styleUrl: './grid-canvas.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
  })
  export class GridCanvasComponent implements AfterViewInit, OnDestroy {
    @ViewChild('gridCanvas', { static: true }) canvasRef!: ElementRef<HTMLCanvasElement>;
    @ViewChild('cellEditor', { static: true }) editorRef!: ElementRef<HTMLInputElement>;
    @ViewChild('container', { static: true }) containerRef!: ElementRef<HTMLDivElement>;
  
    private gridService = inject(GridWasmService);
    private zone = inject(NgZone);
  
    private renderer!: CanvasRenderer;
    private rafId = 0;
    private resizeObserver!: ResizeObserver;
  
    // Editor state
    editorVisible = false;
    editorStyle: Record<string, string> = {};
    editorValue = '';
  
    // Touch scroll
    private touchStartX = 0;
    private touchStartY = 0;
    private scrollStartX = 0;
    private scrollStartY = 0;
  
    ngAfterViewInit(): void {
      this.zone.runOutsideAngular(() => {
        this.initAsync();
      });
    }
  
    private async initAsync(): Promise<void> {
      const container = this.containerRef.nativeElement;
      const rect = container.getBoundingClientRect();
  
      await this.gridService.initialize(rect.width, rect.height);
  
      this.renderer = new CanvasRenderer(this.canvasRef.nativeElement);
      this.renderer.resize(rect.width, rect.height);
  
      this.setupResizeObserver();
      this.setupCanvasEvents();
      this.scheduleRender();
    }
  
    private setupResizeObserver(): void {
      this.resizeObserver = new ResizeObserver((entries) => {
        for (const entry of entries) {
          const { width, height } = entry.contentRect;
          this.gridService.setViewport(width, height);
          this.renderer.resize(width, height);
          this.scheduleRender();
        }
      });
      this.resizeObserver.observe(this.containerRef.nativeElement);
    }
  
    private setupCanvasEvents(): void {
      const canvas = this.canvasRef.nativeElement;
  
      // Wheel scroll
      canvas.addEventListener('wheel', (e: WheelEvent) => {
        e.preventDefault();
        this.gridService.scrollBy(e.deltaX, e.deltaY);
        if (this.editorVisible) this.hideEditor();
        this.scheduleRender();
      }, { passive: false });
  
      // Click
      canvas.addEventListener('click', (e: MouseEvent) => {
        const rect = canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
  
        const hit = this.gridService.hitTest(x, y);
        if (!hit) return;
  
        switch (hit.type) {
          case 'cell':
            if (this.editorVisible) this.commitEdit();
            this.gridService.select(hit.row!, hit.col!);
            this.scheduleRender();
            break;
          case 'row_group':
          case 'col_group':
            this.gridService.toggleGroup(hit.group_id!);
            this.scheduleRender();
            break;
          case 'col_header':
            // Select entire column: just highlight header
            break;
        }
      });
  
      // Double-click to edit
      canvas.addEventListener('dblclick', (e: MouseEvent) => {
        const rect = canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
  
        const hit = this.gridService.hitTest(x, y);
        if (hit?.type === 'cell') {
          this.showEditor(hit.row!, hit.col!);
        }
      });
  
      // Mouse move for resize cursor
      canvas.addEventListener('mousemove', (e: MouseEvent) => {
        if (this.gridService.isResizing()) {
          this.gridService.updateColResize(e.clientX);
          this.scheduleRender();
          return;
        }
  
        const rect = canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        const hit = this.gridService.hitTest(x, y);
  
        canvas.style.cursor = hit?.type === 'col_resize' ? 'col-resize' : 'default';
      });
  
      // Mouse down for column resize
      canvas.addEventListener('mousedown', (e: MouseEvent) => {
        const rect = canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        const hit = this.gridService.hitTest(x, y);
  
        if (hit?.type === 'col_resize') {
          e.preventDefault();
          this.gridService.startColResize(hit.col!, e.clientX);
        }
      });
  
      // Mouse up
      window.addEventListener('mouseup', () => {
        if (this.gridService.isResizing()) {
          this.gridService.endColResize();
          this.scheduleRender();
        }
      });
  
      // Touch events
      canvas.addEventListener('touchstart', (e: TouchEvent) => {
        const t = e.touches[0];
        this.touchStartX = t.clientX;
        this.touchStartY = t.clientY;
        this.scrollStartX = this.gridService.getScrollX();
        this.scrollStartY = this.gridService.getScrollY();
      });
  
      canvas.addEventListener('touchmove', (e: TouchEvent) => {
        e.preventDefault();
        const t = e.touches[0];
        const dx = this.touchStartX - t.clientX;
        const dy = this.touchStartY - t.clientY;
        this.gridService.setScroll(this.scrollStartX + dx, this.scrollStartY + dy);
        this.scheduleRender();
      }, { passive: false });
    }
  
    // ─── Keyboard ──────────────────────────────────────────────
  
    @HostListener('window:keydown', ['$event'])
    onKeyDown(e: KeyboardEvent): void {
      if (this.editorVisible) return;
  
      const sel = this.gridService.selectedCell();
      if (!sel) return;
  
      let handled = true;
      switch (e.key) {
        case 'ArrowUp':    this.gridService.moveSelection(-1, 0); break;
        case 'ArrowDown':  this.gridService.moveSelection(1, 0); break;
        case 'ArrowLeft':  this.gridService.moveSelection(0, -1); break;
        case 'ArrowRight': this.gridService.moveSelection(0, 1); break;
        case 'Tab':
          this.gridService.moveSelection(0, e.shiftKey ? -1 : 1);
          break;
        case 'Enter':
        case 'F2':
          this.showEditor(sel.row, sel.col);
          break;
        case 'Delete':
        case 'Backspace':
          this.gridService.clearCell(sel.row, sel.col);
          break;
        default:
          // Start typing to edit
          if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
            this.showEditor(sel.row, sel.col, '');
            // Let the key through to the input
            return;
          }
          handled = false;
      }
  
      if (handled) {
        e.preventDefault();
        this.scheduleRender();
      }
    }
  
    // ─── Editor ────────────────────────────────────────────────
  
    private showEditor(row: number, col: number, initialValue?: string): void {
      this.gridService.select(row, col);
      this.gridService.startEdit(row, col);
  
      const rect = this.gridService.cellScreenRect(row, col);
      if (!rect) return;
  
      this.editorValue = initialValue ?? this.gridService.getCellText(row, col);
      this.editorStyle = {
        left: rect.x + 'px',
        top: rect.y + 'px',
        width: rect.w + 'px',
        height: rect.h + 'px',
      };
      this.editorVisible = true;
      this.scheduleRender();
  
      // Focus after Angular change detection
      setTimeout(() => {
        const input = this.editorRef.nativeElement;
        input.focus();
        if (initialValue === undefined) {
          input.select();
        } else {
          input.setSelectionRange(input.value.length, input.value.length);
        }
      }, 0);
    }
  
    private hideEditor(): void {
      this.editorVisible = false;
      this.gridService.stopEdit();
    }
  
    commitEdit(): void {
      const row = this.gridService.getSelectedRow();
      const col = this.gridService.getSelectedCol();
      if (row >= 0 && col >= 0) {
        this.gridService.setCell(row, col, this.editorValue);
      }
      this.hideEditor();
      this.scheduleRender();
    }
  
    cancelEdit(): void {
      this.hideEditor();
      this.scheduleRender();
    }
  
    onEditorKeydown(e: KeyboardEvent): void {
      if (e.key === 'Enter') {
        e.preventDefault();
        this.commitEdit();
        this.gridService.moveSelection(1, 0);
        this.scheduleRender();
      } else if (e.key === 'Escape') {
        this.cancelEdit();
      } else if (e.key === 'Tab') {
        e.preventDefault();
        this.commitEdit();
        this.gridService.moveSelection(0, e.shiftKey ? -1 : 1);
        this.scheduleRender();
      }
    }
  
    // ─── Render loop ───────────────────────────────────────────
  
    scheduleRender(): void {
      if (this.rafId) return;
      this.rafId = requestAnimationFrame(() => {
        this.rafId = 0;
        this.draw();
      });
    }
  
    private draw(): void {
      const frame = this.gridService.renderFrame();
      if (frame) {
        this.renderer.draw(frame);
      }
    }
  
    // ─── Public method for toolbar ─────────────────────────────
  
    triggerRender(): void {
      this.scheduleRender();
    }
  
    ngOnDestroy(): void {
      if (this.rafId) cancelAnimationFrame(this.rafId);
      this.resizeObserver?.disconnect();
    }
  }
  