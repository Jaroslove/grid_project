import type { Frame, VB, VH, VC, SB, Met } from '../models/grid.models';

export class CanvasRenderer {
  private ctx: CanvasRenderingContext2D;
  private dpr = 1;
  private readonly T = {
    bg:'#0F172A', border:'#1e293b',
    hBg:'#1a2744', hBorder:'#334155', hTxt:'#64748B', hHi:'#334155',
    selB:'#3B82F6',
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

  draw(f: Frame): void {
    const c = this.ctx;
    const W = this.canvas.width / this.dpr, H = this.canvas.height / this.dpr;
    const m = f.m;
    const dR = W - m.sbs, dB = H - m.sbs;
    const cbh = m.cbd * m.bs; // column bracket height

    c.fillStyle = this.T.bg; c.fillRect(0, 0, W, H);

    // data cells
    c.save(); c.beginPath(); c.rect(m.ox+m.fw, m.oy+m.fh, dR-m.ox-m.fw, dB-m.oy-m.fh); c.clip();
    for (const cl of f.cells) if (cl.col>=m.fc && cl.row>=m.fr) this.dc(c,cl);
    c.restore();
    // frozen cols
    c.save(); c.beginPath(); c.rect(m.ox, m.oy+m.fh, m.fw, dB-m.oy-m.fh); c.clip();
    for (const cl of f.cells) if (cl.col<m.fc && cl.row>=m.fr) this.dc(c,cl);
    c.restore();
    // frozen rows
    c.save(); c.beginPath(); c.rect(m.ox+m.fw, m.oy, dR-m.ox-m.fw, m.fh); c.clip();
    for (const cl of f.cells) if (cl.row<m.fr && cl.col>=m.fc) this.dc(c,cl);
    c.restore();
    // frozen corner
    c.save(); c.beginPath(); c.rect(m.ox, m.oy, m.fw, m.fh); c.clip();
    for (const cl of f.cells) if (cl.row<m.fr && cl.col<m.fc) this.dc(c,cl);
    c.restore();

    // frozen pane lines
    if (m.fc>0){c.strokeStyle=this.T.fLine;c.lineWidth=1.5;c.beginPath();c.moveTo(m.ox+m.fw+.5,m.oy);c.lineTo(m.ox+m.fw+.5,dB);c.stroke();}
    if (m.fr>0){c.strokeStyle=this.T.fLine;c.lineWidth=1.5;c.beginPath();c.moveTo(m.ox,m.oy+m.fh+.5);c.lineTo(dR,m.oy+m.fh+.5);c.stroke();}

    // col headers (A, B, C...)
    c.save(); c.beginPath(); c.rect(m.ox, cbh, dR-m.ox, m.chh); c.clip();
    for (const h of f.ch) {
      c.fillStyle = h.hi ? this.T.hHi : this.T.hBg;
      c.fillRect(h.pos, cbh, h.sz, m.chh);
      c.strokeStyle=this.T.hBorder; c.lineWidth=.5; c.strokeRect(h.pos+.5,cbh+.5,h.sz-1,m.chh-1);
      c.fillStyle=this.T.hTxt; c.font="500 9px 'Inter',sans-serif"; c.textAlign='center'; c.textBaseline='middle';
      c.fillText(h.lbl, h.pos+h.sz/2, cbh+m.chh/2+.5);
      c.fillStyle=this.T.hBorder; c.fillRect(h.pos+h.sz-1.5, cbh+3, 1.5, m.chh-6);
    }
    c.restore();

    // row headers
    const rhx = m.ox - m.rhw;
    c.save(); c.beginPath(); c.rect(rhx, m.oy+m.fh, m.rhw, dB-m.oy-m.fh); c.clip();
    for (const h of f.rh) {
      c.fillStyle = h.hi ? this.T.hHi : this.T.hBg;
      c.fillRect(rhx, h.pos, m.rhw, h.sz);
      c.strokeStyle=this.T.hBorder; c.lineWidth=.5; c.strokeRect(rhx+.5,h.pos+.5,m.rhw-1,h.sz-1);
      c.fillStyle=this.T.hTxt; c.font="400 9px 'Inter',sans-serif"; c.textAlign='center'; c.textBaseline='middle';
      c.fillText(h.lbl, rhx+m.rhw/2, h.pos+h.sz/2+.5);
    }
    c.restore();

    // row brackets
    c.save(); c.beginPath(); c.rect(0, m.oy+m.fh, rhx, dB-m.oy-m.fh); c.clip();
    for (const b of f.rb) this.bracketV(c, b, m.bs);
    c.restore();

    // col brackets
    if (cbh > 0) {
      c.save(); c.beginPath(); c.rect(m.ox+m.fw, 0, dR-m.ox-m.fw, cbh); c.clip();
      for (const b of f.cb) this.bracketH(c, b, m.bs);
      c.restore();
    }

    // corner
    c.fillStyle=this.T.corner; c.fillRect(0,0,m.ox,m.oy);
    c.strokeStyle=this.T.hBorder; c.lineWidth=.5; c.strokeRect(0,0,m.ox,m.oy);

    // scrollbars
    this.sb(c, f.hs, false); this.sb(c, f.vs, true);
    c.fillStyle=this.T.sbCorner; c.fillRect(W-m.sbs, H-m.sbs, m.sbs, m.sbs);
  }

  private dc(c: CanvasRenderingContext2D, cl: VC): void {
    const {sx,sy,w,h}=cl;
    c.fillStyle=cl.bg; c.fillRect(sx,sy,w,h);
    c.strokeStyle=this.T.border; c.lineWidth=.5; c.strokeRect(sx+.5,sy+.5,w-1,h-1);
    if(cl.ct===4){c.strokeStyle=this.T.gtB;c.lineWidth=1;c.strokeRect(sx+.5,sy+.5,w-1,h-1);c.strokeStyle='#FBBF24';c.lineWidth=1.5;c.beginPath();c.moveTo(sx,sy+.5);c.lineTo(sx+w,sy+.5);c.stroke();}
    else if(cl.ct===3){c.strokeStyle=this.T.stB;c.lineWidth=.5;c.strokeRect(sx+.5,sy+.5,w-1,h-1);c.strokeStyle='#93C5FD40';c.lineWidth=1;c.beginPath();c.moveTo(sx,sy+.5);c.lineTo(sx+w,sy+.5);c.stroke();}
    if(cl.text&&!cl.edit){
      c.save();c.beginPath();c.rect(sx+2,sy+1,w-4,h-2);c.clip();c.fillStyle=cl.fg;
      const fw=cl.bold?'600':'400',fs=cl.ct===4?'12px':cl.ct===1?'10px':'11px';
      c.font=`${fw} ${fs} 'Inter',-apple-system,sans-serif`;c.textBaseline='middle';const ty=sy+h/2+.5,p=6;
      if(cl.ta===2){c.textAlign='right';c.fillText(cl.text,sx+w-p,ty);}
      else if(cl.ta===1){c.textAlign='center';c.fillText(cl.text,sx+w/2,ty);}
      else{c.textAlign='left';c.fillText(cl.text,sx+p+(cl.ind||0),ty);}
      c.restore();
    }
    if(cl.sel){c.strokeStyle=this.T.selB;c.lineWidth=2;c.strokeRect(sx+1,sy+1,w-2,h-2);}
  }

  private bracketV(c: CanvasRenderingContext2D, b: VB, bs: number): void {
    const bx=b.d*bs, mid=bx+bs/2, s=b.s, e=b.e;
    if(e<=s+4)return;
    c.strokeStyle=this.T.bLine;c.lineWidth=1;
    c.beginPath();c.moveTo(mid,s+7);c.lineTo(mid,e-7);c.stroke();
    c.beginPath();c.moveTo(mid,s+7);c.lineTo(mid+4,s+7);c.stroke();
    c.beginPath();c.moveTo(mid,e-7);c.lineTo(mid+4,e-7);c.stroke();
    const by=Math.max(s+1,Math.min((s+e)/2-6,e-14));
    this.btn(c,mid-6,by,b.collapsed);
  }

  private bracketH(c: CanvasRenderingContext2D, b: VB, bs: number): void {
    const by=b.d*bs, mid=by+bs/2, s=b.s, e=b.e;
    if(e<=s+4)return;
    c.strokeStyle=this.T.bLine;c.lineWidth=1;
    c.beginPath();c.moveTo(s+7,mid);c.lineTo(e-7,mid);c.stroke();
    c.beginPath();c.moveTo(s+7,mid);c.lineTo(s+7,mid+4);c.stroke();
    c.beginPath();c.moveTo(e-7,mid);c.lineTo(e-7,mid+4);c.stroke();
    const bx=Math.max(s+1,Math.min((s+e)/2-6,e-14));
    this.btn(c,bx,mid-6,b.collapsed);
  }

  private sb(c: CanvasRenderingContext2D, s: SB, vert: boolean): void {
    if(!s.vis)return;
    c.fillStyle=this.T.sbTrack;c.fillRect(s.tx,s.ty,s.tw,s.th);
    c.strokeStyle=this.T.hBorder;c.lineWidth=.5;
    if(vert){c.beginPath();c.moveTo(s.tx+.5,s.ty);c.lineTo(s.tx+.5,s.ty+s.th);c.stroke();}
    else{c.beginPath();c.moveTo(s.tx,s.ty+.5);c.lineTo(s.tx+s.tw,s.ty+.5);c.stroke();}
    const p=2,r=3;
    c.fillStyle=this.T.sbThumb;c.beginPath();c.roundRect(s.bx+p,s.by+p,s.bw-p*2,s.bh-p*2,r);c.fill();
  }

  private btn(c: CanvasRenderingContext2D, x: number, y: number, collapsed: boolean): void {
    const s=12,r=2;
    c.fillStyle=this.T.bBtn;c.beginPath();c.roundRect(x,y,s,s,r);c.fill();
    c.strokeStyle=this.T.bLine;c.lineWidth=.5;c.beginPath();c.roundRect(x,y,s,s,r);c.stroke();
    c.fillStyle=this.T.bTxt;c.font="bold 10px 'Inter',monospace";c.textAlign='center';c.textBaseline='middle';
    c.fillText(collapsed?'+':'−',x+s/2,y+s/2+.5);
  }
}
