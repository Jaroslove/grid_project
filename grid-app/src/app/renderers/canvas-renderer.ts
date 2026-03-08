import type { RenderFrame, VBracket, VHeader, VCell, SBar, Metrics } from '../models/grid.models';

export class CanvasRenderer {
  private ctx: CanvasRenderingContext2D;
  private dpr = 1;

  private readonly T = {
    bg:'#0F172A', border:'#1e293b',
    hBg:'#1a2744', hBorder:'#334155', hTxt:'#64748B', hHi:'#334155',
    selB:'#3B82F6', selBg:'#1E3A5F',
    bLine:'#475569', bBtn:'#334155', bTxt:'#E2E8F0',
    corner:'#0F172A', fLine:'#3B82F680',
    sbTrack:'#0c1322', sbThumb:'#334155', sbCorner:'#0c1322',
    gtB:'#FBBF2440', stB:'#93C5FD30',
  };

  constructor(private canvas: HTMLCanvasElement) {
    this.ctx = canvas.getContext('2d', { alpha: false })!;
    this.dpr = devicePixelRatio || 1;
  }

  resize(w: number, h: number): void {
    this.dpr = devicePixelRatio || 1;
    this.canvas.width = w * this.dpr; this.canvas.height = h * this.dpr;
    this.canvas.style.width = w+'px'; this.canvas.style.height = h+'px';
    this.ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
  }

  draw(f: RenderFrame): void {
    const c = this.ctx;
    const W = this.canvas.width / this.dpr, H = this.canvas.height / this.dpr;
    const m = f.m; const ox = m.ox, oy = m.oy, fw = m.fw, fh = m.fh, sbs = m.sbs;
    const dR = W - sbs, dB = H - sbs;

    c.fillStyle = this.T.bg; c.fillRect(0, 0, W, H);

    // data cells
    c.save(); c.beginPath(); c.rect(ox+fw, oy+fh, dR-ox-fw, dB-oy-fh); c.clip();
    for (const cl of f.cells) { if (cl.col >= m.fc && cl.row >= m.fr) this.dc(c, cl); }
    c.restore();

    // frozen cols
    c.save(); c.beginPath(); c.rect(ox, oy+fh, fw, dB-oy-fh); c.clip();
    for (const cl of f.cells) { if (cl.col < m.fc && cl.row >= m.fr) this.dc(c, cl); }
    c.restore();

    // frozen rows
    c.save(); c.beginPath(); c.rect(ox+fw, oy, dR-ox-fw, fh); c.clip();
    for (const cl of f.cells) { if (cl.row < m.fr && cl.col >= m.fc) this.dc(c, cl); }
    c.restore();

    // frozen corner
    c.save(); c.beginPath(); c.rect(ox, oy, fw, fh); c.clip();
    for (const cl of f.cells) { if (cl.row < m.fr && cl.col < m.fc) this.dc(c, cl); }
    c.restore();

    // frozen lines
    if (m.fc > 0) { c.strokeStyle=this.T.fLine; c.lineWidth=1.5; c.beginPath(); c.moveTo(ox+fw+.5,oy); c.lineTo(ox+fw+.5,dB); c.stroke(); }
    if (m.fr > 0) { c.strokeStyle=this.T.fLine; c.lineWidth=1.5; c.beginPath(); c.moveTo(ox,oy+fh+.5); c.lineTo(dR,oy+fh+.5); c.stroke(); }

    // col headers
    c.save(); c.beginPath(); c.rect(ox,0,dR-ox,oy); c.clip();
    for (const h of f.ch) {
      c.fillStyle = h.highlighted ? this.T.hHi : this.T.hBg;
      c.fillRect(h.pos,0,h.size,m.chh);
      c.strokeStyle=this.T.hBorder; c.lineWidth=.5; c.strokeRect(h.pos+.5,.5,h.size-1,m.chh-1);
      c.fillStyle=this.T.hTxt; c.font="500 9px 'Inter',sans-serif"; c.textAlign='center'; c.textBaseline='middle';
      c.fillText(h.label, h.pos+h.size/2, m.chh/2+.5);
      c.fillStyle=this.T.hBorder; c.fillRect(h.pos+h.size-1.5, 3, 1.5, m.chh-6);
    }
    c.restore();

    // row headers
    const rhx = ox - m.rhw;
    c.save(); c.beginPath(); c.rect(rhx, oy+fh, m.rhw, dB-oy-fh); c.clip();
    for (const h of f.rh) {
      c.fillStyle = h.highlighted ? this.T.hHi : this.T.hBg;
      c.fillRect(rhx, h.pos, m.rhw, h.size);
      c.strokeStyle=this.T.hBorder; c.lineWidth=.5; c.strokeRect(rhx+.5, h.pos+.5, m.rhw-1, h.size-1);
      c.fillStyle=this.T.hTxt; c.font="400 9px 'Inter',sans-serif"; c.textAlign='center'; c.textBaseline='middle';
      c.fillText(h.label, rhx+m.rhw/2, h.pos+h.size/2+.5);
    }
    c.restore();

    // brackets
    c.save(); c.beginPath(); c.rect(0, oy+fh, rhx, dB-oy-fh); c.clip();
    for (const b of f.rb) {
      const bx = b.depth * m.bs, mid = bx + m.bs/2;
      const s = Math.max(b.start, oy+fh), e = Math.min(b.end, dB);
      if (e <= s+4) continue;
      c.strokeStyle=this.T.bLine; c.lineWidth=1;
      c.beginPath(); c.moveTo(mid, s+7); c.lineTo(mid, e-7); c.stroke();
      c.beginPath(); c.moveTo(mid, s+7); c.lineTo(mid+4, s+7); c.stroke();
      c.beginPath(); c.moveTo(mid, e-7); c.lineTo(mid+4, e-7); c.stroke();
      const by = Math.max(s+1, Math.min((s+e)/2-6, e-14));
      this.btn(c, mid-6, by, b.collapsed);
    }
    c.restore();

    // corner
    c.fillStyle=this.T.corner; c.fillRect(0,0,ox,oy);
    c.strokeStyle=this.T.hBorder; c.lineWidth=.5; c.strokeRect(0,0,ox,oy);

    // scrollbars
    this.sb(c, f.hs, false); this.sb(c, f.vs, true);
    c.fillStyle=this.T.sbCorner; c.fillRect(W-sbs, H-sbs, sbs, sbs);
  }

  private dc(c: CanvasRenderingContext2D, cl: VCell): void {
    const {sx,sy,w,h} = cl;
    c.fillStyle = cl.bg; c.fillRect(sx,sy,w,h);
    c.strokeStyle=this.T.border; c.lineWidth=.5; c.strokeRect(sx+.5,sy+.5,w-1,h-1);

    if (cl.cell_type === 4) { // grand total
      c.strokeStyle=this.T.gtB; c.lineWidth=1; c.strokeRect(sx+.5,sy+.5,w-1,h-1);
      c.strokeStyle='#FBBF24'; c.lineWidth=1.5; c.beginPath(); c.moveTo(sx,sy+.5); c.lineTo(sx+w,sy+.5); c.stroke();
    } else if (cl.cell_type === 3) { // subtotal
      c.strokeStyle=this.T.stB; c.lineWidth=.5; c.strokeRect(sx+.5,sy+.5,w-1,h-1);
      c.strokeStyle='#93C5FD40'; c.lineWidth=1; c.beginPath(); c.moveTo(sx,sy+.5); c.lineTo(sx+w,sy+.5); c.stroke();
    }

    if (cl.text && !cl.editing) {
      c.save(); c.beginPath(); c.rect(sx+2,sy+1,w-4,h-2); c.clip();
      c.fillStyle=cl.fg;
      const fw = cl.bold ? '600' : '400';
      const fs = cl.cell_type===4?'12px':cl.cell_type===1?'10px':'11px';
      c.font = `${fw} ${fs} 'Inter',-apple-system,sans-serif`;
      c.textBaseline='middle'; const ty = sy+h/2+.5; const p = 6;
      if (cl.text_align===2) { c.textAlign='right'; c.fillText(cl.text, sx+w-p, ty); }
      else if (cl.text_align===1) { c.textAlign='center'; c.fillText(cl.text, sx+w/2, ty); }
      else { c.textAlign='left'; c.fillText(cl.text, sx+p+(cl.indent||0), ty); }
      c.restore();
    }
    if (cl.selected) { c.strokeStyle=this.T.selB; c.lineWidth=2; c.strokeRect(sx+1,sy+1,w-2,h-2); }
  }

  private sb(c: CanvasRenderingContext2D, s: SBar, vert: boolean): void {
    if (!s.visible) return;
    c.fillStyle=this.T.sbTrack; c.fillRect(s.tx,s.ty,s.tw,s.th);
    c.strokeStyle=this.T.hBorder; c.lineWidth=.5;
    if (vert) { c.beginPath(); c.moveTo(s.tx+.5,s.ty); c.lineTo(s.tx+.5,s.ty+s.th); c.stroke(); }
    else { c.beginPath(); c.moveTo(s.tx,s.ty+.5); c.lineTo(s.tx+s.tw,s.ty+.5); c.stroke(); }
    const p = 2, r = 3;
    c.fillStyle=this.T.sbThumb; c.beginPath(); c.roundRect(s.bx+p,s.by+p,s.bw-p*2,s.bh-p*2,r); c.fill();
  }

  private btn(c: CanvasRenderingContext2D, x: number, y: number, col: boolean): void {
    const s=12,r=2;
    c.fillStyle=this.T.bBtn; c.beginPath(); c.roundRect(x,y,s,s,r); c.fill();
    c.strokeStyle=this.T.bLine; c.lineWidth=.5; c.beginPath(); c.roundRect(x,y,s,s,r); c.stroke();
    c.fillStyle=this.T.bTxt; c.font="bold 10px 'Inter',monospace"; c.textAlign='center'; c.textBaseline='middle';
    c.fillText(col?'+':'−', x+s/2, y+s/2+.5);
  }
}
