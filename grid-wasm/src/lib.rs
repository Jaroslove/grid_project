use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, BTreeMap, BTreeSet};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum AggFunc { Sum, Count, Average, Min, Max }
impl AggFunc {
    fn apply(&self, v: &[f64]) -> f64 {
        if v.is_empty() { return 0.0; }
        match self {
            AggFunc::Sum => v.iter().sum(),
            AggFunc::Count => v.len() as f64,
            AggFunc::Average => v.iter().sum::<f64>() / v.len() as f64,
            AggFunc::Min => v.iter().cloned().fold(f64::INFINITY, f64::min),
            AggFunc::Max => v.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        }
    }
    fn label(&self) -> &str {
        match self { AggFunc::Sum=>"Sum", AggFunc::Count=>"Count", AggFunc::Average=>"Avg", AggFunc::Min=>"Min", AggFunc::Max=>"Max" }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PivotField { pub name: String }
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ValueField { pub name: String, pub agg: AggFunc, pub label: Option<String> }
impl ValueField {
    fn display_label(&self) -> String { self.label.clone().unwrap_or_else(|| format!("{} of {}", self.agg.label(), self.name)) }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PivotConfig {
    pub row_fields: Vec<PivotField>,
    pub col_fields: Vec<PivotField>,
    pub value_fields: Vec<ValueField>,
    pub show_row_subtotals: bool,
    pub show_col_subtotals: bool,
    pub show_grand_totals: bool,
}

#[derive(Clone, Debug)]
struct Rec { s: HashMap<String, String>, n: HashMap<String, f64> }
#[derive(Clone, Debug)]
struct Node {
    key: String, fk: Vec<String>, depth: usize, children: Vec<usize>,
    is_sub: bool, is_gt: bool, collapsed: bool, grid_idx: Option<u32>,
}
#[derive(Clone, Debug, Default)]
struct Cell {
    text: String, bg: Option<String>, fg: Option<String>,
    bold: bool, is_hdr: bool, is_sub: bool, is_gt: bool, is_val: bool,
}

type AggIdx = HashMap<(String, String), Vec<Vec<f64>>>;

// ── render types ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct VC {
    pub sx: f64, pub sy: f64, pub w: f64, pub h: f64,
    pub text: String, pub bg: String, pub fg: String,
    pub bold: bool, pub row: u32, pub col: u32,
    pub sel: bool, pub edit: bool,
    pub ct: u8, pub ta: u8, pub ind: f64,
}
#[derive(Serialize)]
pub struct VH { pub pos: f64, pub sz: f64, pub lbl: String, pub idx: u32, pub hi: bool }
#[derive(Serialize)]
pub struct VB { pub id: u32, pub lbl: String, pub s: f64, pub e: f64, pub d: u32, pub collapsed: bool }
#[derive(Serialize)]
pub struct SB { pub vis: bool, pub tx: f64, pub ty: f64, pub tw: f64, pub th: f64, pub bx: f64, pub by: f64, pub bw: f64, pub bh: f64 }
#[derive(Serialize)]
pub struct Met {
    pub ox: f64, pub oy: f64, pub fw: f64, pub fh: f64,
    pub fc: u32, pub fr: u32, pub rhw: f64, pub chh: f64,
    pub rbd: u32, pub cbd: u32, pub bs: f64, pub sbs: f64,
}
#[derive(Serialize)]
pub struct Frame {
    pub cells: Vec<VC>, pub ch: Vec<VH>, pub rh: Vec<VH>,
    pub rb: Vec<VB>, pub cb: Vec<VB>,
    pub m: Met, pub hs: SB, pub vs: SB,
}

// ── Hit result with FULL field names ─────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct Hit {
    #[serde(rename = "type")]
    pub hit_type: String,
    pub row: Option<u32>,
    pub col: Option<u32>,
    pub key: Option<String>,
}

fn make_hit(t: &str, row: Option<u32>, col: Option<u32>, key: Option<String>) -> String {
    serde_json::to_string(&Hit {
        hit_type: t.into(), row, col, key,
    }).unwrap()
}

// ── Grid ─────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct Grid {
    recs: Vec<Rec>, cfg: Option<PivotConfig>,
    cells: HashMap<(u32,u32), Cell>,
    cw: BTreeMap<u32,f64>, rh: BTreeMap<u32,f64>,
    dcw: f64, drh: f64,
    sx: f64, sy: f64, vw: f64, vh: f64,
    rn: Vec<Node>, rr: Vec<usize>,
    cn: Vec<Node>, cr: Vec<usize>,
    tr: u32, tc: u32, fc: u32, fr: u32,
    crow: BTreeSet<String>, ccol: BTreeSet<String>,
    cntw: f64, cnth: f64, frzw: f64, frzh: f64, sz_dirty: bool,
    sel_r: i32, sel_c: i32, ed_r: i32, ed_c: i32,
    rhw: f64, chh: f64, bs: f64, sbs: f64, mt: f64,
    rcol: i32, rsx: f64, rsw: f64,
    dh: bool, dv: bool, doff: f64,
    dirty: bool,
}

#[wasm_bindgen]
impl Grid {
    #[wasm_bindgen(constructor)]
    pub fn new(w: f64, h: f64) -> Self {
        Self {
            recs:vec![], cfg:None,
            cells:HashMap::new(), cw:BTreeMap::new(), rh:BTreeMap::new(),
            dcw:90.0, drh:24.0,
            sx:0.0, sy:0.0, vw:w, vh:h,
            rn:vec![], rr:vec![], cn:vec![], cr:vec![],
            tr:0, tc:0, fc:0, fr:0,
            crow:BTreeSet::new(), ccol:BTreeSet::new(),
            cntw:0.0, cnth:0.0, frzw:0.0, frzh:0.0, sz_dirty:true,
            sel_r:-1, sel_c:-1, ed_r:-1, ed_c:-1,
            rhw:40.0, chh:20.0, bs:18.0, sbs:12.0, mt:20.0,
            rcol:-1, rsx:0.0, rsw:0.0,
            dh:false, dv:false, doff:0.0,
            dirty:false,
        }
    }

    pub fn load_data(&mut self, json: &str) {
        if let Ok(raw) = serde_json::from_str::<Vec<HashMap<String, serde_json::Value>>>(json) {
            self.recs = raw.into_iter().map(|m| {
                let mut s = HashMap::new(); let mut n = HashMap::new();
                for (k, v) in m {
                    match v {
                        serde_json::Value::String(sv) => { s.insert(k, sv); }
                        serde_json::Value::Number(nv) => {
                            if let Some(f) = nv.as_f64() { n.insert(k.clone(), f); }
                            s.insert(k, nv.to_string());
                        }
                        _ => { s.insert(k, v.to_string()); }
                    }
                }
                Rec { s, n }
            }).collect();
            self.dirty = true; self.sz_dirty = true;
        }
    }

    pub fn set_pivot_config(&mut self, json: &str) {
        if let Ok(c) = serde_json::from_str::<PivotConfig>(json) {
            self.cfg = Some(c); self.dirty = true; self.sz_dirty = true;
        }
    }

    pub fn build_pivot(&mut self) {
        let cfg = match &self.cfg { Some(c) => c.clone(), None => return };
        if self.recs.is_empty() { return; }

        self.cells.clear();
        self.rn.clear(); self.rr.clear();
        self.cn.clear(); self.cr.clear();
        self.sel_r = -1; self.sel_c = -1; self.ed_r = -1; self.ed_c = -1;

        let nv = cfg.value_fields.len().max(1) as u32;

        let (mut rnodes, rroots) = build_tree(&self.recs, &cfg.row_fields, cfg.show_row_subtotals, cfg.show_grand_totals, &self.crow);
        let (mut cnodes, croots) = build_tree(&self.recs, &cfg.col_fields, cfg.show_col_subtotals, cfg.show_grand_totals, &self.ccol);

        self.fc = cfg.row_fields.len().max(1) as u32;
        self.fr = if cfg.col_fields.is_empty() { 1 } else { cfg.col_fields.len() as u32 + 1 };

        let mut ri = self.fr;
        assign_rows(&mut rnodes, &rroots, &mut ri);
        let mut ci = self.fc;
        assign_cols(&mut cnodes, &croots, nv, &mut ci);

        self.tr = ri; self.tc = ci;
        self.rn = rnodes; self.rr = rroots;
        self.cn = cnodes; self.cr = croots;

        write_headers(&cfg, self.fr, &mut self.cells);
        write_row_labels(&self.rn, self.fc, &mut self.cells);
        write_col_labels(&self.cn, &cfg, self.fr, &mut self.cells);
        let agg = build_agg_index(&self.recs, &cfg);
        write_data(&self.rn, &self.cn, &self.cr, &cfg, &agg, &mut self.cells);
        style_cells(&mut self.cells, self.fr, self.fc);
        init_widths(&mut self.cw, self.fc, self.tc, self.dcw);

        self.dirty = false; self.sz_dirty = true;
        self.recomp(); self.clamp();
    }

    fn recomp(&mut self) {
        if !self.sz_dirty { return; }
        self.frzw = (0..self.fc).map(|c| self.colw(c)).sum();
        self.frzh = (0..self.fr).map(|r| self.rowh(r)).sum();
        self.cntw = (self.fc..self.tc).map(|c| self.colw(c)).sum();
        self.cnth = (self.fr..self.tr).map(|r| self.rowh(r)).sum();
        self.sz_dirty = false;
    }

    fn dvw(&self) -> f64 { (self.vw - self.ox() - self.frzw - self.sbs).max(1.0) }
    fn dvh(&self) -> f64 { (self.vh - self.oy() - self.frzh - self.sbs).max(1.0) }
    fn msx(&self) -> f64 { (self.cntw - self.dvw()).max(0.0) }
    fn msy(&self) -> f64 { (self.cnth - self.dvh()).max(0.0) }
    fn clamp(&mut self) {
        self.sx = self.sx.max(0.0).min(self.msx());
        self.sy = self.sy.max(0.0).min(self.msy());
    }

    fn colw(&self, c: u32) -> f64 { *self.cw.get(&c).unwrap_or(&self.dcw) }
    fn rowh(&self, r: u32) -> f64 { *self.rh.get(&r).unwrap_or(&self.drh) }

    fn rbd(&self) -> u32 {
        self.rn.iter().filter(|n| !n.children.is_empty() && !n.is_sub && !n.is_gt)
            .map(|n| n.depth as u32 + 1).max().unwrap_or(0)
    }
    fn cbd(&self) -> u32 {
        self.cn.iter().filter(|n| !n.children.is_empty() && !n.is_sub && !n.is_gt)
            .map(|n| n.depth as u32 + 1).max().unwrap_or(0)
    }

    fn ox(&self) -> f64 { self.rbd() as f64 * self.bs + self.rhw }
    fn oy(&self) -> f64 { self.cbd() as f64 * self.bs + self.chh }

    fn scol_px(&self, c: u32) -> f64 { (self.fc..c).map(|i| self.colw(i)).sum() }
    fn srow_px(&self, r: u32) -> f64 { (self.fr..r).map(|i| self.rowh(i)).sum() }
    fn fcol_px(&self, c: u32) -> f64 { (0..c.min(self.fc)).map(|i| self.colw(i)).sum() }
    fn frow_px(&self, r: u32) -> f64 { (0..r.min(self.fr)).map(|i| self.rowh(i)).sum() }

    fn last_row_desc(&self, i: usize) -> Option<u32> {
        let n = &self.rn[i];
        if n.children.is_empty() || n.collapsed { return n.grid_idx; }
        let mut last = n.grid_idx;
        for &ci in &n.children {
            if let Some(l) = self.last_row_desc(ci) {
                match last { Some(p) if l > p => last = Some(l), None => last = Some(l), _ => {} }
            }
        }
        last
    }

    fn last_col_desc(&self, i: usize) -> Option<u32> {
        let n = &self.cn[i];
        if n.children.is_empty() || n.collapsed { return n.grid_idx; }
        let mut last = n.grid_idx;
        let nv = self.cfg.as_ref().map(|c| c.value_fields.len().max(1) as u32).unwrap_or(1);
        for &ci in &n.children {
            if let Some(l) = self.last_col_desc(ci) {
                let adj = if self.cn[ci].children.is_empty() || self.cn[ci].collapsed { l + nv - 1 } else { l };
                match last { Some(p) if adj > p => last = Some(adj), None => last = Some(adj), _ => {} }
            }
        }
        last
    }

    fn hsb(&self) -> SB {
        let dvw = self.dvw();
        if self.cntw <= dvw { return SB{vis:false,tx:0.0,ty:0.0,tw:0.0,th:0.0,bx:0.0,by:0.0,bw:0.0,bh:0.0}; }
        let tx = self.ox(); let ty = self.vh - self.sbs;
        let tw = self.vw - tx - self.sbs; let th = self.sbs;
        let r = dvw / self.cntw;
        let bw = (tw*r).max(self.mt).min(tw);
        let ms = self.msx();
        let f = if ms > 0.0 { self.sx/ms } else { 0.0 };
        SB{vis:true,tx,ty,tw,th,bx:tx+f*(tw-bw),by:ty,bw,bh:th}
    }

    fn vsb(&self) -> SB {
        let dvh = self.dvh();
        if self.cnth <= dvh { return SB{vis:false,tx:0.0,ty:0.0,tw:0.0,th:0.0,bx:0.0,by:0.0,bw:0.0,bh:0.0}; }
        let tx = self.vw - self.sbs; let ty = self.oy();
        let tw = self.sbs; let th = self.vh - ty - self.sbs;
        let r = dvh / self.cnth;
        let bh = (th*r).max(self.mt).min(th);
        let ms = self.msy();
        let f = if ms > 0.0 { self.sy/ms } else { 0.0 };
        SB{vis:true,tx,ty,tw,th,bx:tx,by:ty+f*(th-bh),bw:tw,bh}
    }

    fn ensure(&mut self) { if self.dirty { self.build_pivot(); } self.recomp(); }

    // ─── public api ──────────────────────────────────────────

    pub fn set_viewport(&mut self, w: f64, h: f64) { self.vw=w; self.vh=h; self.recomp(); self.clamp(); }
    pub fn scroll_by(&mut self, dx: f64, dy: f64) { self.ensure(); self.sx+=dx; self.sy+=dy; self.clamp(); }
    pub fn set_scroll(&mut self, x: f64, y: f64) { self.ensure(); self.sx=x; self.sy=y; self.clamp(); }
    pub fn get_scroll_x(&self) -> f64 { self.sx }
    pub fn get_scroll_y(&self) -> f64 { self.sy }

    pub fn select(&mut self, r: i32, c: i32) { self.sel_r=r; self.sel_c=c; }
    pub fn sel_row(&self) -> i32 { self.sel_r }
    pub fn sel_col(&self) -> i32 { self.sel_c }
    pub fn edit(&mut self, r: i32, c: i32) { self.ed_r=r; self.ed_c=c; }
    pub fn edit_row(&self) -> i32 { self.ed_r }
    pub fn edit_col(&self) -> i32 { self.ed_c }

    pub fn set_cell(&mut self, r: u32, c: u32, t: &str) { self.cells.entry((r,c)).or_default().text=t.into(); }
    pub fn get_cell_text(&self, r: u32, c: u32) -> String { self.cells.get(&(r,c)).map(|d|d.text.clone()).unwrap_or_default() }
    pub fn clear_cell(&mut self, r: u32, c: u32) { self.cells.remove(&(r,c)); }
    pub fn set_col_width(&mut self, c: u32, w: f64) { self.cw.insert(c, w.max(30.0)); self.sz_dirty=true; }

    pub fn move_selection(&mut self, dr: i32, dc: i32) {
        self.ensure();
        if self.sel_r<0{self.sel_r=0;} if self.sel_c<0{self.sel_c=0;}
        self.sel_r=(self.sel_r+dr).max(0).min(self.tr as i32-1);
        self.sel_c=(self.sel_c+dc).max(0).min(self.tc as i32-1);
        self.ensure_vis(self.sel_r as u32, self.sel_c as u32);
    }
    fn ensure_vis(&mut self, r: u32, c: u32) {
        self.recomp();
        if c >= self.fc {
            let cs=self.scol_px(c); let ce=cs+self.colw(c); let vw=self.dvw();
            if cs<self.sx{self.sx=cs;} else if ce>self.sx+vw{self.sx=ce-vw;}
        }
        if r >= self.fr {
            let rs=self.srow_px(r); let re=rs+self.rowh(r); let vh=self.dvh();
            if rs<self.sy{self.sy=rs;} else if re>self.sy+vh{self.sy=re-vh;}
        }
        self.clamp();
    }

    pub fn start_col_resize(&mut self, c: i32, x: f64) {
        if c>=0{self.rcol=c;self.rsx=x;self.rsw=self.colw(c as u32);}
    }
    pub fn update_col_resize(&mut self, x: f64) {
        if self.rcol>=0{self.cw.insert(self.rcol as u32,(self.rsw+x-self.rsx).max(30.0));self.sz_dirty=true;}
    }
    pub fn end_col_resize(&mut self) { self.rcol=-1; self.recomp(); self.clamp(); }
    pub fn is_resizing(&self) -> bool { self.rcol>=0 }

    pub fn toggle_row_collapse(&mut self, k: &str) {
        let s=k.to_string(); if !self.crow.remove(&s){self.crow.insert(s);} self.dirty=true; self.sz_dirty=true;
    }
    pub fn toggle_col_collapse(&mut self, k: &str) {
        let s=k.to_string(); if !self.ccol.remove(&s){self.ccol.insert(s);} self.dirty=true; self.sz_dirty=true;
    }

    pub fn start_h_drag(&mut self, x: f64) { let s=self.hsb(); if s.vis&&x>=s.bx&&x<=s.bx+s.bw{self.dh=true;self.doff=x-s.bx;} }
    pub fn start_v_drag(&mut self, y: f64) { let s=self.vsb(); if s.vis&&y>=s.by&&y<=s.by+s.bh{self.dv=true;self.doff=y-s.by;} }
    pub fn update_h_drag(&mut self, x: f64) {
        if !self.dh{return;} let s=self.hsb(); let sp=s.tw-s.bw;
        if sp<=0.0{return;} let nt=(x-self.doff).max(s.tx).min(s.tx+sp);
        self.sx=((nt-s.tx)/sp)*self.msx(); self.clamp();
    }
    pub fn update_v_drag(&mut self, y: f64) {
        if !self.dv{return;} let s=self.vsb(); let sp=s.th-s.bh;
        if sp<=0.0{return;} let nt=(y-self.doff).max(s.ty).min(s.ty+sp);
        self.sy=((nt-s.ty)/sp)*self.msy(); self.clamp();
    }
    pub fn end_drag(&mut self) { self.dh=false; self.dv=false; }
    pub fn is_dragging_scrollbar(&self) -> bool { self.dh||self.dv }
    pub fn click_h_track(&mut self, x: f64) { let s=self.hsb(); if s.vis{self.sx=((x-s.tx)/s.tw*self.msx()).max(0.0);self.clamp();} }
    pub fn click_v_track(&mut self, y: f64) { let s=self.vsb(); if s.vis{self.sy=((y-s.ty)/s.th*self.msy()).max(0.0);self.clamp();} }

    // ─── hit test ────────────────────────────────────────────

    pub fn hit_test(&mut self, cx: f64, cy: f64) -> String {
        self.ensure();
        let ox = self.ox(); let oy = self.oy();
        let fw = self.frzw; let fh = self.frzh;
        let rbd = self.rbd();
        let cbd = self.cbd();
        let rbw = rbd as f64 * self.bs;  // row bracket total width
        let cbh = cbd as f64 * self.bs;  // col bracket total height

        // ── scrollbars (highest priority) ──
        let hs = self.hsb();
        if hs.vis && cy>=hs.ty && cy<=hs.ty+hs.th && cx>=hs.tx && cx<=hs.tx+hs.tw {
            return make_hit("h_scrollbar", None, None, None);
        }
        let vs = self.vsb();
        if vs.vis && cx>=vs.tx && cx<=vs.tx+vs.tw && cy>=vs.ty && cy<=vs.ty+vs.th {
            return make_hit("v_scrollbar", None, None, None);
        }

        // ── row brackets (left gutter, below col brackets + col header) ──
        // Row brackets occupy x: [0, rbw), y: [oy + fh, vh - sbs)
        if rbw > 0.0 && cx < rbw && cy >= oy + fh && cy < self.vh - self.sbs {
            let depth = (cx / self.bs) as u32;
            // Convert screen Y to scrollable-row data space
            let data_y = (cy - oy - fh) + self.sy;

            for (i, n) in self.rn.iter().enumerate() {
                if n.depth as u32 != depth { continue; }
                if n.children.is_empty() || n.is_sub || n.is_gt { continue; }
                if let Some(gr) = n.grid_idx {
                    if gr < self.fr { continue; }
                    let top = self.srow_px(gr);
                    let lr = self.last_row_desc(i).unwrap_or(gr);
                    let bot = self.srow_px(lr) + self.rowh(lr);
                    if data_y >= top && data_y <= bot {
                        return make_hit("row_bracket", Some(i as u32), None, Some(n.fk.join("|")));
                    }
                }
            }
        }

        // ── column brackets (top gutter, right of row brackets + row header) ──
        // Col brackets occupy x: [ox + fw, vw - sbs), y: [0, cbh)
        if cbh > 0.0 && cy < cbh && cx >= ox + fw && cx < self.vw - self.sbs {
            let depth = (cy / self.bs) as u32;
            // Convert screen X to scrollable-col data space
            let data_x = (cx - ox - fw) + self.sx;

            for (i, n) in self.cn.iter().enumerate() {
                if n.depth as u32 != depth { continue; }
                if n.children.is_empty() || n.is_sub || n.is_gt { continue; }
                if let Some(gc) = n.grid_idx {
                    if gc < self.fc { continue; }
                    let left = self.scol_px(gc);
                    let nv = self.cfg.as_ref().map(|c| c.value_fields.len().max(1) as u32).unwrap_or(1);
                    let lr = self.last_col_desc(i).unwrap_or(gc);
                    // End pixel: last leaf col + its width(s)
                    let right = if self.cn[i].children.is_empty() || self.cn[i].collapsed {
                        self.scol_px(gc + nv)
                    } else {
                        self.scol_px(lr + 1)
                    };
                    if data_x >= left && data_x <= right {
                        return make_hit("col_bracket", None, Some(i as u32), Some(n.fk.join("|")));
                    }
                }
            }
        }

        // ── cells ──
        if cx >= ox && cy >= oy && cx < self.vw - self.sbs && cy < self.vh - self.sbs {
            let rx = cx - ox;
            let ry = cy - oy;
            let in_fc = rx < fw;
            let in_fr = ry < fh;

            let col = if in_fc {
                px_to_fc(rx, self.fc, &self.cw, self.dcw)
            } else {
                px_to_sc(rx - fw + self.sx, self.fc, self.tc, &self.cw, self.dcw)
            };
            let row = if in_fr {
                px_to_fr(ry, self.fr, &self.rh, self.drh)
            } else {
                px_to_sr(ry - fh + self.sy, self.fr, self.tr, &self.rh, self.drh)
            };

            // col resize handle
            if in_fr || row < self.fr {
                let ce = if in_fc {
                    self.fcol_px(col) + self.colw(col)
                } else {
                    self.scol_px(col) + self.colw(col) - self.sx + fw
                };
                if (rx - ce).abs() < 5.0 {
                    return make_hit("col_resize", None, Some(col), None);
                }
            }

            return make_hit("cell", Some(row), Some(col), None);
        }

        make_hit("none", None, None, None)
    }

    pub fn cell_screen_rect(&mut self, r: u32, c: u32) -> String {
        self.ensure();
        let ox=self.ox(); let oy=self.oy();
        let sx = if c<self.fc { self.fcol_px(c)+ox } else { self.scol_px(c)-self.sx+ox+self.frzw };
        let sy = if r<self.fr { self.frow_px(r)+oy } else { self.srow_px(r)-self.sy+oy+self.frzh };
        format!(r#"{{"x":{},"y":{},"w":{},"h":{}}}"#, sx, sy, self.colw(c), self.rowh(r))
    }

    // ─── render ──────────────────────────────────────────────

    pub fn render_frame(&mut self) -> String {
        self.ensure();
        let ox=self.ox(); let oy=self.oy();
        let fw=self.frzw; let fh=self.frzh;
        let dvw=self.dvw(); let dvh=self.dvh();

        let mut cells=Vec::new();
        let mut chv=Vec::new();
        let mut rhv=Vec::new();

        let fcols:Vec<(u32,f64)>={(||{let mut v=Vec::new();let mut px=0.0;for c in 0..self.fc{v.push((c,ox+px));px+=self.colw(c);}v})()};
        let scols:Vec<(u32,f64)>={(||{let mut v=Vec::new();let mut px=0.0_f64;
            for c in self.fc..self.tc{let w=self.colw(c);let s=px-self.sx;
                if s+w>0.0&&s<dvw{v.push((c,ox+fw+s));}px+=w;if s>dvw{break;}}v})()};
        let frows:Vec<(u32,f64)>={(||{let mut v=Vec::new();let mut py=0.0;for r in 0..self.fr{v.push((r,oy+py));py+=self.rowh(r);}v})()};
        let srows:Vec<(u32,f64)>={(||{let mut v=Vec::new();let mut py=0.0_f64;
            for r in self.fr..self.tr{let h=self.rowh(r);let s=py-self.sy;
                if s+h>0.0&&s<dvh{v.push((r,oy+fh+s));}py+=h;if s>dvh{break;}}v})()};

        for &(r,sy) in frows.iter().chain(srows.iter()) {
            let h=self.rowh(r);
            for &(c,sx) in fcols.iter().chain(scols.iter()) {
                self.emit(&mut cells, r, c, sx, sy, self.colw(c), h);
            }
        }

        for &(c,sx) in fcols.iter().chain(scols.iter()) {
            chv.push(VH{pos:sx,sz:self.colw(c),lbl:col_label(c),idx:c,hi:self.sel_c==c as i32});
        }
        for &(r,sy) in &srows {
            rhv.push(VH{pos:sy,sz:self.rowh(r),lbl:(r+1).to_string(),idx:r,hi:self.sel_r==r as i32});
        }

        let rb = self.vis_row_brackets(oy, fh, dvh);
        let cb = self.vis_col_brackets(ox, fw, dvw);

        let m = Met{ox,oy,fw,fh,fc:self.fc,fr:self.fr,rhw:self.rhw,chh:self.chh,
            rbd:self.rbd(),cbd:self.cbd(),bs:self.bs,sbs:self.sbs};

        let frame = Frame{cells,ch:chv,rh:rhv,rb,cb,m,hs:self.hsb(),vs:self.vsb()};
        serde_json::to_string(&frame).unwrap()
    }

    fn emit(&self, cells: &mut Vec<VC>, r: u32, c: u32, sx: f64, sy: f64, w: f64, h: f64) {
        let cd = self.cells.get(&(r,c));
        let sel = self.sel_r==r as i32 && self.sel_c==c as i32;
        let ed = self.ed_r==r as i32 && self.ed_c==c as i32;
        let (ct,ta,ind) = if let Some(d) = cd {
            let ct = if d.is_gt{4u8} else if d.is_sub{3} else if d.is_hdr{1} else if d.is_val{2} else {0};
            let ta = if d.is_val{2u8} else if d.is_hdr&&c<self.fc{0} else if d.is_hdr{1} else {0};
            let ind = if c<self.fc && !d.is_sub && !d.is_gt && r>=self.fr {
                self.rn.iter().find(|n|n.grid_idx==Some(r)).map(|n|n.depth as f64*12.0).unwrap_or(0.0)
            } else {0.0};
            (ct,ta,ind)
        } else {(0u8,0u8,0.0)};
        let bg = if ed{"#FFFFFF".into()} else if sel{"#1E3A5F".into()} else {cd.and_then(|d|d.bg.clone()).unwrap_or("#0F172A".into())};
        let fg = cd.and_then(|d|d.fg.clone()).unwrap_or("#E2E8F0".into());
        cells.push(VC{sx,sy,w,h,text:cd.map(|d|d.text.clone()).unwrap_or_default(),bg,fg,
            bold:cd.map(|d|d.bold).unwrap_or(false),row:r,col:c,sel,edit:ed,ct,ta,ind});
    }

    fn vis_row_brackets(&self, oy: f64, fh: f64, dvh: f64) -> Vec<VB> {
        let mut out = Vec::new();
        for (i,n) in self.rn.iter().enumerate() {
            if n.children.is_empty() || n.is_sub || n.is_gt { continue; }
            if let Some(gr) = n.grid_idx {
                if gr < self.fr { continue; }
                let lr = self.last_row_desc(i).unwrap_or(gr);
                let sp = self.srow_px(gr) - self.sy + oy + fh;
                let ep = self.srow_px(lr) + self.rowh(lr) - self.sy + oy + fh;
                if ep > oy+fh && sp < oy+fh+dvh {
                    out.push(VB{id:i as u32,lbl:n.key.clone(),s:sp,e:ep,d:n.depth as u32,collapsed:n.collapsed});
                }
            }
        }
        out
    }

    fn vis_col_brackets(&self, ox: f64, fw: f64, dvw: f64) -> Vec<VB> {
        let mut out = Vec::new();
        let nv = self.cfg.as_ref().map(|c| c.value_fields.len().max(1) as u32).unwrap_or(1);
        for (i,n) in self.cn.iter().enumerate() {
            if n.children.is_empty() || n.is_sub || n.is_gt { continue; }
            if let Some(gc) = n.grid_idx {
                if gc < self.fc { continue; }
                let lr = self.last_col_desc(i).unwrap_or(gc);
                let sp = self.scol_px(gc) - self.sx + ox + fw;
                let ep = if n.children.is_empty() || n.collapsed {
                    self.scol_px(gc + nv) - self.sx + ox + fw
                } else {
                    self.scol_px(lr + 1) - self.sx + ox + fw
                };
                if ep > ox+fw && sp < ox+fw+dvw {
                    out.push(VB{id:i as u32,lbl:n.key.clone(),s:sp,e:ep,d:n.depth as u32,collapsed:n.collapsed});
                }
            }
        }
        out
    }

    pub fn cell_count(&self) -> u32 { self.cells.len() as u32 }
    pub fn get_total_rows(&self) -> u32 { self.tr }
    pub fn get_total_cols(&self) -> u32 { self.tc }
}

// ── free functions ───────────────────────────────────────────

fn build_tree(recs: &[Rec], fields: &[PivotField], subs: bool, grand: bool, collapsed: &BTreeSet<String>) -> (Vec<Node>, Vec<usize>) {
    let mut nodes = Vec::new();
    if fields.is_empty() {
        nodes.push(Node{key:String::new(),fk:vec![],depth:0,children:vec![],is_sub:false,is_gt:false,collapsed:false,grid_idx:None});
        let mut roots = vec![0usize];
        if grand {
            let gi=nodes.len();
            nodes.push(Node{key:"Grand Total".into(),fk:vec!["__gt__".into()],depth:0,children:vec![],is_sub:false,is_gt:true,collapsed:false,grid_idx:None});
            roots.push(gi);
        }
        return (nodes, roots);
    }
    let mut roots = tree_lvl(recs, fields, &mut nodes, subs, collapsed, 0, &[], &[]);
    if grand {
        let gi=nodes.len();
        nodes.push(Node{key:"Grand Total".into(),fk:vec!["__gt__".into()],depth:0,children:vec![],is_sub:false,is_gt:true,collapsed:false,grid_idx:None});
        roots.push(gi);
    }
    (nodes, roots)
}

fn tree_lvl(recs: &[Rec], fields: &[PivotField], nodes: &mut Vec<Node>, subs: bool, collapsed: &BTreeSet<String>,
    depth: usize, pkey: &[String], filter: &[(usize,String)]) -> Vec<usize>
{
    if depth>=fields.len(){return vec![];}
    let fname = &fields[depth].name;
    let mut uniq:BTreeSet<String> = BTreeSet::new();
    'o: for r in recs {
        for &(fi,ref fv) in filter { if r.s.get(&fields[fi].name).map(|s|s.as_str()).unwrap_or("")!=fv.as_str(){continue 'o;} }
        if let Some(v)=r.s.get(fname){uniq.insert(v.clone());}
    }
    let mut out=Vec::new();
    for val in &uniq {
        let mut fk=pkey.to_vec(); fk.push(val.clone());
        let ks=fk.join("|"); let is_c=collapsed.contains(&ks);
        let ni=nodes.len();
        nodes.push(Node{key:val.clone(),fk:fk.clone(),depth,children:vec![],is_sub:false,is_gt:false,collapsed:is_c,grid_idx:None});
        let mut nf=filter.to_vec(); nf.push((depth,val.clone()));
        let ch=tree_lvl(recs,fields,nodes,subs,collapsed,depth+1,&fk,&nf);
        nodes[ni].children=ch;
        out.push(ni);
        if subs && depth<fields.len()-1 && !nodes[ni].children.is_empty() {
            let si=nodes.len();
            let mut sk=fk; sk.push("__sub__".into());
            nodes.push(Node{key:format!("{} Total",val),fk:sk,depth,children:vec![],is_sub:true,is_gt:false,collapsed:false,grid_idx:None});
            out.push(si);
        }
    }
    out
}

fn assign_rows(nodes: &mut Vec<Node>, ids: &[usize], next: &mut u32) {
    for &i in ids { nodes[i].grid_idx=Some(*next); *next+=1;
        if !nodes[i].collapsed && !nodes[i].children.is_empty() { let ch=nodes[i].children.clone(); assign_rows(nodes,&ch,next); } }
}
fn assign_cols(nodes: &mut Vec<Node>, ids: &[usize], nv: u32, next: &mut u32) {
    for &i in ids { nodes[i].grid_idx=Some(*next);
        if nodes[i].children.is_empty()||nodes[i].collapsed{*next+=nv;}
        else{let ch=nodes[i].children.clone();assign_cols(nodes,&ch,nv,next);}
    }
}

fn write_headers(cfg:&PivotConfig,fr:u32,cells:&mut HashMap<(u32,u32),Cell>){
    let vr=fr.saturating_sub(1);
    for(i,f)in cfg.row_fields.iter().enumerate(){cells.insert((vr,i as u32),Cell{text:f.name.clone(),bold:true,is_hdr:true,bg:Some("#1a2744".into()),fg:Some("#93C5FD".into()),..Default::default()});}
    for(i,f)in cfg.col_fields.iter().enumerate(){cells.insert((i as u32,0),Cell{text:f.name.clone(),bold:true,is_hdr:true,bg:Some("#1a2744".into()),fg:Some("#93C5FD".into()),..Default::default()});}
}
fn write_row_labels(nodes:&[Node],fc:u32,cells:&mut HashMap<(u32,u32),Cell>){
    for n in nodes{if let Some(gr)=n.grid_idx{
        let tc=if n.is_sub||n.is_gt{0}else{(n.depth as u32).min(fc.saturating_sub(1))};
        cells.insert((gr,tc),Cell{text:n.key.clone(),bold:n.is_sub||n.is_gt||n.depth==0,is_hdr:true,is_sub:n.is_sub,is_gt:n.is_gt,..Default::default()});
    }}
}
fn write_col_labels(nodes:&[Node],cfg:&PivotConfig,fr:u32,cells:&mut HashMap<(u32,u32),Cell>){
    for n in nodes{if let Some(gc)=n.grid_idx{
        cells.insert((n.depth as u32,gc),Cell{text:n.key.clone(),bold:true,is_hdr:true,is_sub:n.is_sub,is_gt:n.is_gt,..Default::default()});
        let leaf=n.children.is_empty()||n.collapsed;
        if leaf&&cfg.value_fields.len()>1{let vlr=fr.saturating_sub(1);for(vi,vf)in cfg.value_fields.iter().enumerate(){cells.insert((vlr,gc+vi as u32),Cell{text:vf.display_label(),bold:true,is_hdr:true,..Default::default()});}}
    }}
}

fn build_agg_index(recs:&[Rec],cfg:&PivotConfig)->AggIdx{
    let nv=cfg.value_fields.len();let mut idx:AggIdx=HashMap::new();
    for rec in recs{
        let rv:Vec<String>=cfg.row_fields.iter().map(|f|rec.s.get(&f.name).cloned().unwrap_or_default()).collect();
        let cv:Vec<String>=cfg.col_fields.iter().map(|f|rec.s.get(&f.name).cloned().unwrap_or_default()).collect();
        let vals:Vec<f64>=cfg.value_fields.iter().map(|vf|rec.n.get(&vf.name).copied().unwrap_or(0.0)).collect();
        let mut rkeys=vec!["__gt__".to_string()];
        for i in 0..rv.len(){rkeys.push(rv[..=i].join("|"));}
        for i in 0..rv.len(){let mut k=rv[..=i].join("|");k.push_str("|__sub__");rkeys.push(k);}
        if cfg.row_fields.is_empty(){rkeys.push(String::new());}
        let mut ckeys=vec!["__gt__".to_string()];
        for i in 0..cv.len(){ckeys.push(cv[..=i].join("|"));}
        for i in 0..cv.len(){let mut k=cv[..=i].join("|");k.push_str("|__sub__");ckeys.push(k);}
        if cfg.col_fields.is_empty(){ckeys.push(String::new());}
        for rk in &rkeys{for ck in &ckeys{let e=idx.entry((rk.clone(),ck.clone())).or_insert_with(||vec![Vec::new();nv]);for(vi,&v)in vals.iter().enumerate(){e[vi].push(v);}}}
    }
    idx
}

fn node_key(n:&Node)->String{if n.is_gt{"__gt__".into()}else if n.fk.is_empty(){String::new()}else{n.fk.join("|")}}

fn write_data(rn:&[Node],cn:&[Node],cr:&[usize],cfg:&PivotConfig,agg:&AggIdx,cells:&mut HashMap<(u32,u32),Cell>){
    let leaves=col_leaves(cn,cr);
    for rnode in rn{let gr=match rnode.grid_idx{Some(r)=>r,None=>continue};let rk=node_key(rnode);
        for lc in &leaves{let gc=match lc.grid_idx{Some(c)=>c,None=>continue};let ck=node_key(lc);
            if let Some(buckets)=agg.get(&(rk.clone(),ck.clone())){
                for(vi,vf)in cfg.value_fields.iter().enumerate(){if vi<buckets.len()&&!buckets[vi].is_empty(){
                    let r=vf.agg.apply(&buckets[vi]);
                    cells.insert((gr,gc+vi as u32),Cell{text:fmt_num(r),is_val:true,is_sub:rnode.is_sub||lc.is_sub,is_gt:rnode.is_gt||lc.is_gt,bold:rnode.is_sub||rnode.is_gt||lc.is_sub||lc.is_gt,..Default::default()});
                }}
            }
        }
    }
}

fn col_leaves(nodes:&[Node],roots:&[usize])->Vec<Node>{let mut o=Vec::new();fn w(n:&[Node],ids:&[usize],o:&mut Vec<Node>){for &i in ids{if n[i].children.is_empty()||n[i].collapsed{o.push(n[i].clone());}else{w(n,&n[i].children,o);}}}w(nodes,roots,&mut o);o}

fn style_cells(cells:&mut HashMap<(u32,u32),Cell>,fr:u32,fc:u32){
    let keys:Vec<(u32,u32)>=cells.keys().cloned().collect();
    for(r,c)in keys{let d=cells.get(&(r,c)).unwrap().clone();let mut s=d;
        if s.is_gt{s.bg=Some("#1a3350".into());s.fg=Some("#FBBF24".into());s.bold=true;}
        else if s.is_sub{s.bg=Some("#172544".into());s.fg=Some("#93C5FD".into());s.bold=true;}
        else if s.is_hdr&&r<fr{s.bg=Some("#1a2744".into());s.fg=Some("#94A3B8".into());}
        else if s.is_hdr{s.bg=Some("#152238".into());s.fg=Some("#CBD5E1".into());}
        else if s.is_val{let dr=r.saturating_sub(fr);s.bg=Some(if dr%2==0{"#0F172A".into()}else{"#131d30".into()});s.fg=Some("#E2E8F0".into());}
        cells.insert((r,c),s);
    }
}
fn init_widths(cw:&mut BTreeMap<u32,f64>,fc:u32,tc:u32,dcw:f64){for c in 0..fc{cw.entry(c).or_insert(130.0);}for c in fc..tc{cw.entry(c).or_insert(dcw);}}

fn px_to_fc(x:f64,fc:u32,cw:&BTreeMap<u32,f64>,dcw:f64)->u32{let mut a=0.0;for c in 0..fc{let w=*cw.get(&c).unwrap_or(&dcw);if a+w>x{return c;}a+=w;}fc.saturating_sub(1)}
fn px_to_fr(y:f64,fr:u32,rh:&BTreeMap<u32,f64>,drh:f64)->u32{let mut a=0.0;for r in 0..fr{let h=*rh.get(&r).unwrap_or(&drh);if a+h>y{return r;}a+=h;}fr.saturating_sub(1)}
fn px_to_sc(x:f64,fc:u32,tc:u32,cw:&BTreeMap<u32,f64>,dcw:f64)->u32{let mut a=0.0;for c in fc..tc{let w=*cw.get(&c).unwrap_or(&dcw);if a+w>x{return c;}a+=w;}tc.saturating_sub(1)}
fn px_to_sr(y:f64,fr:u32,tr:u32,rh:&BTreeMap<u32,f64>,drh:f64)->u32{let mut a=0.0;for r in fr..tr{let h=*rh.get(&r).unwrap_or(&drh);if a+h>y{return r;}a+=h;}tr.saturating_sub(1)}

fn col_label(mut c:u32)->String{let mut s=String::new();loop{s.insert(0,(b'A'+(c%26)as u8)as char);if c<26{break;}c=c/26-1;}s}
fn fmt_num(v:f64)->String{if v==v.floor()&&v.abs()<1e15{let i=v as i64;let neg=i<0;let s=i.unsigned_abs().to_string();let b:Vec<u8>=s.bytes().collect();let mut r=Vec::new();for(j,&ch)in b.iter().enumerate(){if j>0&&(b.len()-j)%3==0{r.push(b',');}r.push(ch);}let f=String::from_utf8(r).unwrap();if neg{format!("-{}",f)}else{f}}else{format!("{:.1}",v)}}