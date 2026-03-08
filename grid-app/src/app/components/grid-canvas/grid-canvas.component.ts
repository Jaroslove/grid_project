import {
  Component, ElementRef, AfterViewInit, OnDestroy,
  ViewChild, NgZone, inject, HostListener, ChangeDetectionStrategy,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { GridWasmService } from '../../services/grid-wasm.service';
import { CanvasRenderer } from '../../renderers/canvas-renderer';
import type { PivotConfig } from '../../models/grid.models';

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

  private svc = inject(GridWasmService);
  private zone = inject(NgZone);
  private renderer!: CanvasRenderer;
  private rafId = 0;
  private resizeObs!: ResizeObserver;

  editorVisible = false;
  editorStyle: Record<string, string> = {};
  editorValue = '';

  private tx0 = 0; private ty0 = 0;
  private sx0 = 0; private sy0 = 0;
  private ltx = 0; private lty = 0; private ltt = 0;
  private vx = 0; private vy = 0;
  private iid = 0;

  ngAfterViewInit(): void { this.zone.runOutsideAngular(() => this.init()); }

  private async init(): Promise<void> {
    const el = this.containerRef.nativeElement;
    const r = el.getBoundingClientRect();
    await this.svc.initialize(r.width, r.height);
    this.renderer = new CanvasRenderer(this.canvasRef.nativeElement);
    this.renderer.resize(r.width, r.height);
    this.setupResize();
    this.setupEvents();
    this.loadData();
    this.scheduleRender();
  }

  private loadData(): void {
    const regions = ['North America','South America','Europe West','Europe East',
      'Asia Pacific','Middle East','Africa','Oceania'];
    const countries: Record<string,string[]> = {
      'North America':['USA','Canada','Mexico'],
      'South America':['Brazil','Argentina','Chile'],
      'Europe West':['UK','France','Germany','Spain','Italy'],
      'Europe East':['Poland','Romania','Hungary'],
      'Asia Pacific':['Japan','South Korea','Singapore','Hong Kong'],
      'Middle East':['UAE','Saudi Arabia','Israel'],
      'Africa':['South Africa','Nigeria','Kenya'],
      'Oceania':['Australia','New Zealand'],
    };
    const cats = ['Electronics','Clothing','Food','Automotive','Healthcare','Sports'];
    const subs: Record<string,string[]> = {
      'Electronics':['Phones','Laptops','Tablets'],
      'Clothing':['Men','Women','Children'],
      'Food':['Snacks','Drinks','Frozen'],
      'Automotive':['Parts','Accessories','Tires'],
      'Healthcare':['Medicine','Supplements','Equipment'],
      'Sports':['Fitness','Outdoor','Team Sports'],
    };
    const months = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
    const quarters = ['Q1','Q2','Q3','Q4'];
    const channels = ['Online','Retail','Wholesale'];

    let seed = 42;
    const rng = () => { seed = (seed * 1103515245 + 12345) & 0x7fffffff; return (seed % 10000) / 10000; };

    const data: any[] = [];
    for (const reg of regions) {
      for (const cty of (countries[reg] || [])) {
        for (const cat of cats) {
          for (const sub of (subs[cat] || [])) {
            for (let mi = 0; mi < 12; mi++) {
              for (const ch of channels) {
                if (rng() < 0.4) continue; // skip 40%
                const sales = Math.round((500 + rng() * 20000) * 100) / 100;
                const qty = Math.max(1, Math.round(sales / (20 + rng() * 100)));
                const cost = Math.round(sales * (0.4 + rng() * 0.3) * 100) / 100;
                data.push({
                  Region: reg, Country: cty, Category: cat, Subcategory: sub,
                  Month: months[mi], Quarter: quarters[Math.floor(mi / 3)],
                  Channel: ch, Sales: sales, Quantity: qty,
                  Profit: Math.round((sales - cost) * 100) / 100,
                });
              }
            }
          }
        }
      }
    }

    console.log(`Generated ${data.length} records`);

    const config: PivotConfig = {
      row_fields: [{ name: 'Region' }, { name: 'Country' }, { name: 'Category' }],
      col_fields: [{ name: 'Quarter' }, { name: 'Channel' }],
      value_fields: [
        { name: 'Sales', agg: 'Sum', label: 'Revenue' },
        { name: 'Quantity', agg: 'Sum', label: 'Units' },
        { name: 'Profit', agg: 'Sum', label: 'Profit' },
      ],
      show_row_subtotals: true,
      show_col_subtotals: true,
      show_grand_totals: true,
    };

    this.svc.loadData(data);
    this.svc.setPivotConfig(config);
    this.svc.buildPivot();
  }

  public loadPivot(data: any[], config: PivotConfig): void {
    this.svc.loadData(data); this.svc.setPivotConfig(config);
    this.svc.buildPivot(); this.scheduleRender();
  }

  private setupResize(): void {
    this.resizeObs = new ResizeObserver(es => {
      for (const e of es) {
        const { width, height } = e.contentRect;
        this.svc.setViewport(width, height);
        this.renderer.resize(width, height);
        this.scheduleRender();
      }
    });
    this.resizeObs.observe(this.containerRef.nativeElement);
  }

  private setupEvents(): void {
    const cv = this.canvasRef.nativeElement;

    cv.addEventListener('wheel', (e: WheelEvent) => {
      e.preventDefault(); this.cancelInertia();
      let dx = e.deltaX, dy = e.deltaY;
      if (e.deltaMode === 1) { dx *= 28; dy *= 28; }
      if (e.shiftKey && Math.abs(dy) > Math.abs(dx)) { dx = dy; dy = 0; }
      this.svc.scrollBy(dx, dy);
      if (this.editorVisible) this.hideEditor();
      this.scheduleRender();
    }, { passive: false });

    cv.addEventListener('mousedown', (e: MouseEvent) => {
      const r = cv.getBoundingClientRect();
      const x = e.clientX - r.left, y = e.clientY - r.top;
      const h = this.svc.hitTest(x, y);
      if (!h) return;
      if (h.type === 'h_scrollbar') { e.preventDefault(); this.svc.startHDrag(x); if (!this.svc.isDraggingScrollbar()) { this.svc.clickHTrack(x); this.scheduleRender(); } return; }
      if (h.type === 'v_scrollbar') { e.preventDefault(); this.svc.startVDrag(y); if (!this.svc.isDraggingScrollbar()) { this.svc.clickVTrack(y); this.scheduleRender(); } return; }
      if (h.type === 'col_resize') { e.preventDefault(); this.svc.startColResize(h.col!, e.clientX); return; }
    });

    cv.addEventListener('click', (e: MouseEvent) => {
      if (this.svc.isResizing() || this.svc.isDraggingScrollbar()) return;
      const r = cv.getBoundingClientRect();
      const h = this.svc.hitTest(e.clientX - r.left, e.clientY - r.top);
      if (!h) return;
      if (h.type === 'cell') { if (this.editorVisible) this.commitEdit(); this.svc.select(h.row!, h.col!); this.scheduleRender(); }
      else if (h.type === 'row_bracket' && h.key) { this.svc.toggleRowCollapse(h.key); this.scheduleRender(); }
    });

    cv.addEventListener('dblclick', (e: MouseEvent) => {
      const r = cv.getBoundingClientRect();
      const h = this.svc.hitTest(e.clientX - r.left, e.clientY - r.top);
      if (h?.type === 'cell') this.showEditor(h.row!, h.col!);
    });

    window.addEventListener('mousemove', (e: MouseEvent) => {
      const r = cv.getBoundingClientRect();
      if (this.svc.isDraggingScrollbar()) {
        this.svc.updateHDrag(e.clientX - r.left);
        this.svc.updateVDrag(e.clientY - r.top);
        this.scheduleRender(); return;
      }
      if (this.svc.isResizing()) { this.svc.updateColResize(e.clientX); this.scheduleRender(); return; }
      const h = this.svc.hitTest(e.clientX - r.left, e.clientY - r.top);
      cv.style.cursor = h?.type === 'col_resize' ? 'col-resize' : h?.type === 'row_bracket' ? 'pointer' : 'default';
    });

    window.addEventListener('mouseup', () => {
      if (this.svc.isDraggingScrollbar()) { this.svc.endDrag(); this.scheduleRender(); }
      if (this.svc.isResizing()) { this.svc.endColResize(); this.scheduleRender(); }
    });

    cv.addEventListener('touchstart', (e: TouchEvent) => {
      this.cancelInertia();
      const t = e.touches[0];
      this.tx0 = t.clientX; this.ty0 = t.clientY;
      this.ltx = t.clientX; this.lty = t.clientY; this.ltt = performance.now();
      this.sx0 = this.svc.getScrollX(); this.sy0 = this.svc.getScrollY();
      this.vx = 0; this.vy = 0;
    });
    cv.addEventListener('touchmove', (e: TouchEvent) => {
      e.preventDefault();
      const t = e.touches[0]; const now = performance.now(); const dt = now - this.ltt;
      if (dt > 0) { const a = 0.3; this.vx = a*((this.ltx-t.clientX)/dt)*16+(1-a)*this.vx; this.vy = a*((this.lty-t.clientY)/dt)*16+(1-a)*this.vy; }
      this.ltx = t.clientX; this.lty = t.clientY; this.ltt = now;
      this.svc.setScroll(this.sx0+this.tx0-t.clientX, this.sy0+this.ty0-t.clientY);
      if (this.editorVisible) this.hideEditor();
      this.scheduleRender();
    }, { passive: false });
    cv.addEventListener('touchend', () => { if (Math.abs(this.vx)>0.5||Math.abs(this.vy)>0.5) this.startInertia(); });
  }

  private startInertia(): void {
    this.cancelInertia();
    const tick = () => {
      this.vx *= 0.95; this.vy *= 0.95;
      if (Math.abs(this.vx)<0.3 && Math.abs(this.vy)<0.3) return;
      this.svc.scrollBy(this.vx, this.vy); this.scheduleRender();
      this.iid = requestAnimationFrame(tick);
    };
    this.iid = requestAnimationFrame(tick);
  }
  private cancelInertia(): void { if (this.iid) { cancelAnimationFrame(this.iid); this.iid=0; } this.vx=0; this.vy=0; }

  @HostListener('window:keydown', ['$event'])
  onKeyDown(e: KeyboardEvent): void {
    if (this.editorVisible) return;
    const s = this.svc.selectedCell(); if (!s) return;
    let handled = true;
    switch (e.key) {
      case 'ArrowUp': this.svc.moveSelection(-1,0); break;
      case 'ArrowDown': this.svc.moveSelection(1,0); break;
      case 'ArrowLeft': this.svc.moveSelection(0,-1); break;
      case 'ArrowRight': this.svc.moveSelection(0,1); break;
      case 'Tab': this.svc.moveSelection(0,e.shiftKey?-1:1); break;
      case 'Enter': case 'F2': this.showEditor(s.row,s.col); break;
      case 'Delete': case 'Backspace': this.svc.clearCell(s.row,s.col); break;
      case 'PageDown': this.svc.scrollBy(0,this.containerRef.nativeElement.clientHeight*0.8); break;
      case 'PageUp': this.svc.scrollBy(0,-this.containerRef.nativeElement.clientHeight*0.8); break;
      case 'Home': if (e.ctrlKey) { this.svc.setScroll(0,0); this.svc.select(0,0); } break;
      default: if (e.key.length===1 && !e.ctrlKey && !e.metaKey) { this.showEditor(s.row,s.col,''); return; } handled=false;
    }
    if (handled) { e.preventDefault(); this.scheduleRender(); }
  }

  private showEditor(r: number, c: number, init?: string): void {
    this.svc.select(r,c); this.svc.startEdit(r,c);
    const rect = this.svc.cellScreenRect(r,c); if (!rect) return;
    this.editorValue = init ?? this.svc.getCellText(r,c);
    this.editorStyle = { left:rect.x+'px', top:rect.y+'px', width:rect.w+'px', height:rect.h+'px' };
    this.editorVisible = true; this.scheduleRender();
    setTimeout(() => { const inp = this.editorRef.nativeElement; inp.focus(); if (init===undefined) inp.select(); else inp.setSelectionRange(inp.value.length,inp.value.length); },0);
  }
  private hideEditor(): void { this.editorVisible = false; this.svc.stopEdit(); }
  commitEdit(): void { const r=this.svc.getSelectedRow(),c=this.svc.getSelectedCol(); if (r>=0&&c>=0) this.svc.setCell(r,c,this.editorValue); this.hideEditor(); this.scheduleRender(); }
  cancelEdit(): void { this.hideEditor(); this.scheduleRender(); }
  onEditorKeydown(e: KeyboardEvent): void {
    if (e.key==='Enter') { e.preventDefault(); this.commitEdit(); this.svc.moveSelection(1,0); this.scheduleRender(); }
    else if (e.key==='Escape') this.cancelEdit();
    else if (e.key==='Tab') { e.preventDefault(); this.commitEdit(); this.svc.moveSelection(0,e.shiftKey?-1:1); this.scheduleRender(); }
  }

  scheduleRender(): void { if (this.rafId) return; this.rafId = requestAnimationFrame(() => { this.rafId=0; this.draw(); }); }
  private draw(): void { const f = this.svc.renderFrame(); if (f) this.renderer.draw(f); }
  triggerRender(): void { this.scheduleRender(); }
  ngOnDestroy(): void { if (this.rafId) cancelAnimationFrame(this.rafId); this.cancelInertia(); this.resizeObs?.disconnect(); }
}
