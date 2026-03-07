import type { RenderFrame, GroupBracket, VisibleHeader } from '../models/grid.models';

export class CanvasRenderer {
  private ctx: CanvasRenderingContext2D;
  private dpr: number;

  // Theme
  private readonly theme = {
    bg: '#0F172A',
    cellBg: '#1E293B',
    cellBorder: '#334155',
    headerBg: '#1E293B',
    headerBorder: '#475569',
    headerText: '#94A3B8',
    headerHighlight: '#334155',
    selectionBorder: '#3B82F6',
    selectionBg: '#1E3A5F',
    bracketLine: '#64748B',
    bracketBtn: '#334155',
    bracketBtnText: '#E2E8F0',
    cornerBg: '#0F172A',
    text: '#E2E8F0',
    textMuted: '#94A3B8',
  };

  constructor(
    private canvas: HTMLCanvasElement
  ) {
    this.ctx = canvas.getContext('2d', { alpha: false })!;
    this.dpr = window.devicePixelRatio || 1;
  }

  resize(width: number, height: number): void {
    this.dpr = window.devicePixelRatio || 1;
    this.canvas.width = width * this.dpr;
    this.canvas.height = height * this.dpr;
    this.canvas.style.width = width + 'px';
    this.canvas.style.height = height + 'px';
    this.ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
  }

  draw(frame: RenderFrame): void {
    const ctx = this.ctx;
    const W = this.canvas.width / this.dpr;
    const H = this.canvas.height / this.dpr;
    const m = frame.metrics;
    const ox = m.content_origin_x;
    const oy = m.content_origin_y;

    // Background
    ctx.fillStyle = this.theme.bg;
    ctx.fillRect(0, 0, W, H);

    // ── Cells ─────────────────────────────────────────────
    ctx.save();
    ctx.beginPath();
    ctx.rect(ox, oy, W - ox, H - oy);
    ctx.clip();

    for (const cell of frame.cells) {
      // Background
      ctx.fillStyle = cell.selected ? this.theme.selectionBg : cell.bg;
      ctx.fillRect(cell.sx, cell.sy, cell.w, cell.h);

      // Border
      ctx.strokeStyle = this.theme.cellBorder;
      ctx.lineWidth = 0.5;
      ctx.strokeRect(cell.sx + 0.5, cell.sy + 0.5, cell.w - 1, cell.h - 1);

      // Text
      if (cell.text && !cell.editing) {
        ctx.save();
        ctx.beginPath();
        ctx.rect(cell.sx + 4, cell.sy + 1, cell.w - 8, cell.h - 2);
        ctx.clip();

        ctx.fillStyle = cell.fg;
        ctx.font = `${cell.bold ? '600' : '400'} 13px 'Inter', sans-serif`;
        ctx.textBaseline = 'middle';
        ctx.textAlign = 'left';
        ctx.fillText(cell.text, cell.sx + 8, cell.sy + cell.h / 2 + 1);
        ctx.restore();
      }

      // Selection ring
      if (cell.selected) {
        ctx.strokeStyle = this.theme.selectionBorder;
        ctx.lineWidth = 2;
        ctx.strokeRect(cell.sx + 1, cell.sy + 1, cell.w - 2, cell.h - 2);
      }
    }
    ctx.restore();

    // ── Column headers ────────────────────────────────────
    ctx.save();
    ctx.beginPath();
    ctx.rect(ox, oy - m.col_header_height, W - ox, m.col_header_height);
    ctx.clip();

    for (const h of frame.col_headers) {
      const y = oy - m.col_header_height;
      ctx.fillStyle = h.highlighted ? this.theme.headerHighlight : this.theme.headerBg;
      ctx.fillRect(h.pos, y, h.size, m.col_header_height);

      ctx.strokeStyle = this.theme.headerBorder;
      ctx.lineWidth = 0.5;
      ctx.strokeRect(h.pos + 0.5, y + 0.5, h.size - 1, m.col_header_height - 1);

      ctx.fillStyle = this.theme.headerText;
      ctx.font = "600 11px 'Inter', sans-serif";
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(h.label, h.pos + h.size / 2, y + m.col_header_height / 2 + 1);

      // Resize handle indicator
      ctx.fillStyle = this.theme.headerBorder;
      ctx.fillRect(h.pos + h.size - 2, y + 4, 2, m.col_header_height - 8);
    }
    ctx.restore();

    // ── Row headers ───────────────────────────────────────
    const rhX = m.group_rows_depth * m.bracket_size;
    ctx.save();
    ctx.beginPath();
    ctx.rect(rhX, oy, m.row_header_width, H - oy);
    ctx.clip();

    for (const h of frame.row_headers) {
      ctx.fillStyle = h.highlighted ? this.theme.headerHighlight : this.theme.headerBg;
      ctx.fillRect(rhX, h.pos, m.row_header_width, h.size);

      ctx.strokeStyle = this.theme.headerBorder;
      ctx.lineWidth = 0.5;
      ctx.strokeRect(rhX + 0.5, h.pos + 0.5, m.row_header_width - 1, h.size - 1);

      ctx.fillStyle = this.theme.headerText;
      ctx.font = "500 11px 'Inter', sans-serif";
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(h.label, rhX + m.row_header_width / 2, h.pos + h.size / 2 + 1);
    }
    ctx.restore();

    // ── Row group brackets ────────────────────────────────
    this.drawRowBrackets(frame.row_brackets, m.bracket_size, oy, H);

    // ── Column group brackets ─────────────────────────────
    this.drawColBrackets(frame.col_brackets, m.bracket_size, ox, W);

    // ── Corner ────────────────────────────────────────────
    ctx.fillStyle = this.theme.cornerBg;
    ctx.fillRect(0, 0, ox, oy);
    ctx.strokeStyle = this.theme.headerBorder;
    ctx.lineWidth = 0.5;
    ctx.strokeRect(0, 0, ox, oy);
  }

  private drawRowBrackets(brackets: GroupBracket[], bracketSize: number, oy: number, H: number): void {
    const ctx = this.ctx;

    for (const b of brackets) {
      const x = b.depth * bracketSize;
      const midX = x + bracketSize / 2;
      const startY = Math.max(b.start, oy);
      const endY = Math.min(b.end, H);
      if (endY <= startY) continue;

      // Vertical line
      ctx.strokeStyle = this.theme.bracketLine;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(midX, startY + 6);
      ctx.lineTo(midX, endY - 6);
      ctx.stroke();

      // Ticks
      ctx.beginPath();
      ctx.moveTo(midX, startY + 6);
      ctx.lineTo(midX + 5, startY + 6);
      ctx.stroke();

      ctx.beginPath();
      ctx.moveTo(midX, endY - 6);
      ctx.lineTo(midX + 5, endY - 6);
      ctx.stroke();

      // Button
      const btnY = Math.max(startY, Math.min((startY + endY) / 2 - 7, endY - 14));
      this.drawGroupButton(midX - 7, btnY, b.collapsed);
    }
  }

  private drawColBrackets(brackets: GroupBracket[], bracketSize: number, ox: number, W: number): void {
    const ctx = this.ctx;

    for (const b of brackets) {
      const y = b.depth * bracketSize;
      const midY = y + bracketSize / 2;
      const startX = Math.max(b.start, ox);
      const endX = Math.min(b.end, W);
      if (endX <= startX) continue;

      ctx.strokeStyle = this.theme.bracketLine;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(startX + 6, midY);
      ctx.lineTo(endX - 6, midY);
      ctx.stroke();

      ctx.beginPath();
      ctx.moveTo(startX + 6, midY);
      ctx.lineTo(startX + 6, midY + 5);
      ctx.stroke();

      ctx.beginPath();
      ctx.moveTo(endX - 6, midY);
      ctx.lineTo(endX - 6, midY + 5);
      ctx.stroke();

      const btnX = Math.max(startX, Math.min((startX + endX) / 2 - 7, endX - 14));
      this.drawGroupButton(btnX, midY - 7, b.collapsed);
    }
  }

  private drawGroupButton(x: number, y: number, collapsed: boolean): void {
    const ctx = this.ctx;
    const size = 14;
    const r = 3;

    // Rounded rect background
    ctx.fillStyle = this.theme.bracketBtn;
    ctx.beginPath();
    ctx.roundRect(x, y, size, size, r);
    ctx.fill();

    ctx.strokeStyle = this.theme.bracketLine;
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    ctx.roundRect(x, y, size, size, r);
    ctx.stroke();

    // Symbol
    ctx.fillStyle = this.theme.bracketBtnText;
    ctx.font = "bold 11px 'Inter', monospace";
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(collapsed ? '+' : '−', x + size / 2, y + size / 2 + 1);
  }
}
