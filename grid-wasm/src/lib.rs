use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, BTreeMap, BTreeSet};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum AggFunc { Sum, Count, Average, Min, Max }

impl AggFunc {
    fn apply(&self, vals: &[f64]) -> f64 {
        if vals.is_empty() { return 0.0; }
        match self {
            AggFunc::Sum => vals.iter().sum(),
            AggFunc::Count => vals.len() as f64,
            AggFunc::Average => vals.iter().sum::<f64>() / vals.len() as f64,
            AggFunc::Min => vals.iter().cloned().fold(f64::INFINITY, f64::min),
            AggFunc::Max => vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        }
    }
    fn label(&self) -> &str {
        match self {
            AggFunc::Sum => "Sum", AggFunc::Count => "Count",
            AggFunc::Average => "Avg", AggFunc::Min => "Min", AggFunc::Max => "Max",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PivotField { pub name: String }

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ValueField {
    pub name: String, pub agg: AggFunc, pub label: Option<String>,
}
impl ValueField {
    fn display_label(&self) -> String {
        self.label.clone().unwrap_or_else(|| format!("{} of {}", self.agg.label(), self.name))
    }
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
struct RawRecord {
    strings: HashMap<String, String>,
    numbers: HashMap<String, f64>,
}

#[derive(Clone, Debug)]
struct PivotNode {
    key: String,
    full_key: Vec<String>,
    depth: usize,
    children: Vec<usize>,
    is_subtotal: bool,
    is_grand_total: bool,
    collapsed: bool,
    grid_index: Option<u32>,
}

#[derive(Clone, Debug, Default)]
struct CellData {
    text: String,
    bg: Option<String>,
    fg: Option<String>,
    bold: bool,
    is_header: bool,
    is_subtotal: bool,
    is_grand_total: bool,
    is_value: bool,
}

// ─── Render types ────────────────────────────────────────────

#[derive(Serialize)]
pub struct VCell {
    pub sx: f64, pub sy: f64, pub w: f64, pub h: f64,
    pub text: String, pub bg: String, pub fg: String,
    pub bold: bool, pub row: u32, pub col: u32,
    pub selected: bool, pub editing: bool,
    pub cell_type: u8,   // 0=empty 1=header 2=value 3=subtotal 4=grand_total
    pub text_align: u8,  // 0=left 1=center 2=right
    pub indent: f64,
}

#[derive(Serialize)]
pub struct VHeader {
    pub pos: f64, pub size: f64, pub label: String,
    pub index: u32, pub highlighted: bool,
}

#[derive(Serialize)]
pub struct VBracket {
    pub id: u32, pub label: String,
    pub start: f64, pub end: f64,
    pub depth: u32, pub collapsed: bool,
}

#[derive(Serialize)]
pub struct SBar {
    pub visible: bool,
    pub tx: f64, pub ty: f64, pub tw: f64, pub th: f64,
    pub bx: f64, pub by: f64, pub bw: f64, pub bh: f64,
}

#[derive(Serialize)]
pub struct Metrics {
    pub ox: f64, pub oy: f64,
    pub fw: f64, pub fh: f64,
    pub fc: u32, pub fr: u32,
    pub rhw: f64, pub chh: f64,
    pub bd: u32, pub bs: f64, pub sbs: f64,
}

#[derive(Serialize)]
pub struct Frame {
    pub cells: Vec<VCell>,
    pub ch: Vec<VHeader>,
    pub rh: Vec<VHeader>,
    pub rb: Vec<VBracket>,
    pub m: Metrics,
    pub hs: SBar,
    pub vs: SBar,
}

#[derive(Serialize, Deserialize)]
pub struct Hit {
    #[serde(rename = "type")]
    pub t: String,
    pub r: Option<u32>,
    pub c: Option<u32>,
    pub k: Option<String>,
}

/// Pre-indexed aggregation bucket.
/// Key = (row_key_joined, col_key_joined)
/// Value = Vec<f64> per value field
type AggIndex = HashMap<(String, String), Vec<Vec<f64>>>;

// ─── Grid ────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct Grid {
    records: Vec<RawRecord>,
    config: Option<PivotConfig>,

    cells: HashMap<(u32, u32), CellData>,
    cw: BTreeMap<u32, f64>,
    rh_map: BTreeMap<u32, f64>,
    dcw: f64, drh: f64,

    sx: f64, sy: f64,
    vw: f64, vh: f64,

    rnodes: Vec<PivotNode>,
    rroots: Vec<usize>,
    cnodes: Vec<PivotNode>,
    croots: Vec<usize>,

    tr: u32, tc: u32,
    fc: u32, fr: u32,

    crow: BTreeSet<String>,
    ccol: BTreeSet<String>,

    // cached
    cntw: f64, cnth: f64,
    frzw: f64, frzh: f64,
    sz_dirty: bool,

    sel_r: i32, sel_c: i32,
    ed_r: i32, ed_c: i32,

    rhw: f64, chh: f64,
    bs: f64, sbs: f64, mt: f64,

    rcol: i32, rsx: f64, rsw: f64,
    dh: bool, dv: bool, doff: f64,

    dirty: bool,
}

#[wasm_bindgen]
impl Grid {
    #[wasm_bindgen(constructor)]
    pub fn new(w: f64, h: f64) -> Self {
        Self {
            records: Vec::new(), config: None,
            cells: HashMap::new(),
            cw: BTreeMap::new(), rh_map: BTreeMap::new(),
            dcw: 90.0, drh: 24.0,
            sx: 0.0, sy: 0.0, vw: w, vh: h,
            rnodes: Vec::new(), rroots: Vec::new(),
            cnodes: Vec::new(), croots: Vec::new(),
            tr: 0, tc: 0, fc: 0, fr: 0,
            crow: BTreeSet::new(), ccol: BTreeSet::new(),
            cntw: 0.0, cnth: 0.0, frzw: 0.0, frzh: 0.0,
            sz_dirty: true,
            sel_r: -1, sel_c: -1, ed_r: -1, ed_c: -1,
            rhw: 40.0, chh: 20.0, bs: 16.0, sbs: 12.0, mt: 20.0,
            rcol: -1, rsx: 0.0, rsw: 0.0,
            dh: false, dv: false, doff: 0.0,
            dirty: false,
        }
    }

    pub fn load_data(&mut self, json: &str) {
        // Parse into optimized format
        if let Ok(raw) = serde_json::from_str::<Vec<HashMap<String, serde_json::Value>>>(json) {
            self.records = raw.into_iter().map(|m| {
                let mut strings = HashMap::new();
                let mut numbers = HashMap::new();
                for (k, v) in m {
                    match v {
                        serde_json::Value::String(s) => { strings.insert(k, s); }
                        serde_json::Value::Number(n) => {
                            if let Some(f) = n.as_f64() {
                                numbers.insert(k.clone(), f);
                            }
                            strings.insert(k, n.to_string());
                        }
                        _ => { strings.insert(k, v.to_string()); }
                    }
                }
                RawRecord { strings, numbers }
            }).collect();
            self.dirty = true; self.sz_dirty = true;
        }
    }

    pub fn set_pivot_config(&mut self, json: &str) {
        if let Ok(c) = serde_json::from_str::<PivotConfig>(json) {
            self.config = Some(c);
            self.dirty = true; self.sz_dirty = true;
        }
    }

    pub fn build_pivot(&mut self) {
        let cfg = match &self.config { Some(c) => c.clone(), None => return };
        if self.records.is_empty() { return; }

        self.cells.clear();
        self.rnodes.clear(); self.rroots.clear();
        self.cnodes.clear(); self.croots.clear();
        self.sel_r = -1; self.sel_c = -1;
        self.ed_r = -1; self.ed_c = -1;

        let nv = cfg.value_fields.len().max(1) as u32;

        // Build trees
        {
            let recs = &self.records;
            let mut nodes = Vec::new();
            let roots = build_tree(recs, &cfg.row_fields, &mut nodes,
                cfg.show_row_subtotals, cfg.show_grand_totals, &self.crow);
            self.rnodes = nodes;
            self.rroots = roots;
        }
        {
            let recs = &self.records;
            let mut nodes = Vec::new();
            let roots = build_tree(recs, &cfg.col_fields, &mut nodes,
                cfg.show_col_subtotals, cfg.show_grand_totals, &self.ccol);
            self.cnodes = nodes;
            self.croots = roots;
        }

        self.fc = cfg.row_fields.len().max(1) as u32;
        self.fr = if cfg.col_fields.is_empty() { 1 } else { cfg.col_fields.len() as u32 + 1 };

        let mut ri = self.fr;
        let rr = self.rroots.clone();
        assign_rows(&mut self.rnodes, &rr, &mut ri);
        let mut ci = self.fc;
        let cr = self.croots.clone();
        assign_cols(&mut self.cnodes, &cr, nv, &mut ci);
        self.tr = ri; self.tc = ci;

        // Write headers
        write_headers(&cfg, self.fr, &mut self.cells);
        write_row_labels(&self.rnodes, self.fc, self.fr, &mut self.cells);
        write_col_labels(&self.cnodes, &cfg, self.fr, &mut self.cells);

        // ── Pre-index records for fast aggregation ──
        let agg_idx = build_agg_index(&self.records, &cfg);

        // ── Write data cells using index ──
        write_data_indexed(&self.rnodes, &self.cnodes, &self.croots,
            &cfg, &agg_idx, &mut self.cells);

        // Style
        style_cells(&mut self.cells, self.fr, self.fc);
        init_col_widths(&mut self.cw, self.fc, self.tc, self.dcw);

        self.dirty = false;
        self.sz_dirty = true;
        self.recompute();
        self.clamp();
    }

    fn recompute(&mut self) {
        if !self.sz_dirty { return; }
        self.frzw = (0..self.fc).map(|c| self.colw(c)).sum();
        self.frzh = (0..self.fr).map(|r| self.rowh(r)).sum();
        self.cntw = (self.fc..self.tc).map(|c| self.colw(c)).sum();
        self.cnth = (self.fr..self.tr).map(|r| self.rowh(r)).sum();
        self.sz_dirty = false;
    }

    fn dvw(&self) -> f64 {
        let o = self.origin_x();
        (self.vw - o - self.frzw - self.sbs).max(1.0)
    }
    fn dvh(&self) -> f64 {
        let o = self.origin_y();
        (self.vh - o - self.frzh - self.sbs).max(1.0)
    }
    fn msx(&self) -> f64 { (self.cntw - self.dvw()).max(0.0) }
    fn msy(&self) -> f64 { (self.cnth - self.dvh()).max(0.0) }
    fn clamp(&mut self) {
        self.sx = self.sx.max(0.0).min(self.msx());
        self.sy = self.sy.max(0.0).min(self.msy());
    }

    fn colw(&self, c: u32) -> f64 { *self.cw.get(&c).unwrap_or(&self.dcw) }
    fn rowh(&self, r: u32) -> f64 { *self.rh_map.get(&r).unwrap_or(&self.drh) }

    fn bdepth(&self) -> u32 {
        self.rnodes.iter()
            .filter(|n| !n.children.is_empty() && !n.is_subtotal && !n.is_grand_total)
            .map(|n| n.depth as u32 + 1).max().unwrap_or(0)
    }

    fn origin_x(&self) -> f64 { self.bdepth() as f64 * self.bs + self.rhw }
    fn origin_y(&self) -> f64 { self.chh }

    fn scol_px(&self, c: u32) -> f64 { (self.fc..c).map(|i| self.colw(i)).sum() }
    fn srow_px(&self, r: u32) -> f64 { (self.fr..r).map(|i| self.rowh(i)).sum() }
    fn fcol_px(&self, c: u32) -> f64 { (0..c.min(self.fc)).map(|i| self.colw(i)).sum() }
    fn frow_px(&self, r: u32) -> f64 { (0..r.min(self.fr)).map(|i| self.rowh(i)).sum() }

    fn last_desc(&self, i: usize) -> Option<u32> {
        let n = &self.rnodes[i];
        if n.children.is_empty() || n.collapsed { return n.grid_index; }
        let mut last = n.grid_index;
        for &ci in &n.children {
            if let Some(l) = self.last_desc(ci) {
                match last { Some(p) if l > p => last = Some(l), None => last = Some(l), _ => {} }
            }
        }
        last
    }

    fn hsb(&self) -> SBar {
        let dvw = self.dvw();
        if self.cntw <= dvw { return SBar { visible:false, tx:0.0,ty:0.0,tw:0.0,th:0.0,bx:0.0,by:0.0,bw:0.0,bh:0.0 }; }
        let ox = self.origin_x();
        let tx = ox; let ty = self.vh - self.sbs;
        let tw = self.vw - ox - self.sbs; let th = self.sbs;
        let r = dvw / self.cntw;
        let bw = (tw * r).max(self.mt).min(tw);
        let ms = self.msx();
        let f = if ms > 0.0 { self.sx / ms } else { 0.0 };
        SBar { visible:true, tx,ty,tw,th, bx: tx + f*(tw-bw), by:ty, bw, bh:th }
    }

    fn vsb(&self) -> SBar {
        let dvh = self.dvh();
        if self.cnth <= dvh { return SBar { visible:false, tx:0.0,ty:0.0,tw:0.0,th:0.0,bx:0.0,by:0.0,bw:0.0,bh:0.0 }; }
        let oy = self.origin_y();
        let tx = self.vw - self.sbs; let ty = oy;
        let tw = self.sbs; let th = self.vh - oy - self.sbs;
        let r = dvh / self.cnth;
        let bh = (th * r).max(self.mt).min(th);
        let ms = self.msy();
        let f = if ms > 0.0 { self.sy / ms } else { 0.0 };
        SBar { visible:true, tx,ty,tw,th, bx:tx, by: ty + f*(th-bh), bw:tw, bh }
    }

    fn ensure(&mut self) {
        if self.dirty { self.build_pivot(); }
        self.recompute();
    }

    // ─── public API ──────────────────────────────────────────

    pub fn set_viewport(&mut self, w: f64, h: f64) { self.vw = w; self.vh = h; self.recompute(); self.clamp(); }

    pub fn scroll_by(&mut self, dx: f64, dy: f64) {
        self.ensure(); self.sx += dx; self.sy += dy; self.clamp();
    }
    pub fn set_scroll(&mut self, x: f64, y: f64) {
        self.ensure(); self.sx = x; self.sy = y; self.clamp();
    }
    pub fn get_scroll_x(&self) -> f64 { self.sx }
    pub fn get_scroll_y(&self) -> f64 { self.sy }

    pub fn select(&mut self, r: i32, c: i32) { self.sel_r = r; self.sel_c = c; }
    pub fn sel_row(&self) -> i32 { self.sel_r }
    pub fn sel_col(&self) -> i32 { self.sel_c }
    pub fn edit(&mut self, r: i32, c: i32) { self.ed_r = r; self.ed_c = c; }
    pub fn edit_row(&self) -> i32 { self.ed_r }
    pub fn edit_col(&self) -> i32 { self.ed_c }

    pub fn set_cell(&mut self, r: u32, c: u32, t: &str) { self.cells.entry((r,c)).or_default().text = t.into(); }
    pub fn get_cell_text(&self, r: u32, c: u32) -> String { self.cells.get(&(r,c)).map(|d| d.text.clone()).unwrap_or_default() }
    pub fn clear_cell(&mut self, r: u32, c: u32) { self.cells.remove(&(r,c)); }

    pub fn set_col_width(&mut self, c: u32, w: f64) { self.cw.insert(c, w.max(30.0)); self.sz_dirty = true; }

    pub fn move_selection(&mut self, dr: i32, dc: i32) {
        self.ensure();
        if self.sel_r < 0 { self.sel_r = 0; }
        if self.sel_c < 0 { self.sel_c = 0; }
        self.sel_r = (self.sel_r+dr).max(0).min(self.tr as i32-1);
        self.sel_c = (self.sel_c+dc).max(0).min(self.tc as i32-1);
        self.ensure_vis(self.sel_r as u32, self.sel_c as u32);
    }

    fn ensure_vis(&mut self, r: u32, c: u32) {
        self.recompute();
        if c >= self.fc {
            let cs = self.scol_px(c); let ce = cs + self.colw(c);
            let vw = self.dvw();
            if cs < self.sx { self.sx = cs; }
            else if ce > self.sx + vw { self.sx = ce - vw; }
        }
        if r >= self.fr {
            let rs = self.srow_px(r); let re = rs + self.rowh(r);
            let vh = self.dvh();
            if rs < self.sy { self.sy = rs; }
            else if re > self.sy + vh { self.sy = re - vh; }
        }
        self.clamp();
    }

    pub fn start_col_resize(&mut self, c: i32, x: f64) {
        if c >= 0 { self.rcol = c; self.rsx = x; self.rsw = self.colw(c as u32); }
    }
    pub fn update_col_resize(&mut self, x: f64) {
        if self.rcol >= 0 {
            self.cw.insert(self.rcol as u32, (self.rsw + x - self.rsx).max(30.0));
            self.sz_dirty = true;
        }
    }
    pub fn end_col_resize(&mut self) { self.rcol = -1; self.recompute(); self.clamp(); }
    pub fn is_resizing(&self) -> bool { self.rcol >= 0 }

    pub fn toggle_row_collapse(&mut self, k: &str) {
        let s = k.to_string();
        if !self.crow.remove(&s) { self.crow.insert(s); }
        self.dirty = true; self.sz_dirty = true;
    }
    pub fn toggle_col_collapse(&mut self, k: &str) {
        let s = k.to_string();
        if !self.ccol.remove(&s) { self.ccol.insert(s); }
        self.dirty = true; self.sz_dirty = true;
    }

    // scrollbar drag
    pub fn start_h_drag(&mut self, x: f64) {
        let s = self.hsb();
        if s.visible && x >= s.bx && x <= s.bx + s.bw { self.dh = true; self.doff = x - s.bx; }
    }
    pub fn start_v_drag(&mut self, y: f64) {
        let s = self.vsb();
        if s.visible && y >= s.by && y <= s.by + s.bh { self.dv = true; self.doff = y - s.by; }
    }
    pub fn update_h_drag(&mut self, x: f64) {
        if !self.dh { return; }
        let s = self.hsb(); let sp = s.tw - s.bw;
        if sp <= 0.0 { return; }
        let nt = (x - self.doff).max(s.tx).min(s.tx + sp);
        self.sx = ((nt - s.tx) / sp) * self.msx(); self.clamp();
    }
    pub fn update_v_drag(&mut self, y: f64) {
        if !self.dv { return; }
        let s = self.vsb(); let sp = s.th - s.bh;
        if sp <= 0.0 { return; }
        let nt = (y - self.doff).max(s.ty).min(s.ty + sp);
        self.sy = ((nt - s.ty) / sp) * self.msy(); self.clamp();
    }
    pub fn end_drag(&mut self) { self.dh = false; self.dv = false; }
    pub fn is_dragging_scrollbar(&self) -> bool { self.dh || self.dv }
    pub fn click_h_track(&mut self, x: f64) {
        let s = self.hsb();
        if s.visible { self.sx = ((x-s.tx)/s.tw * self.msx()).max(0.0); self.clamp(); }
    }
    pub fn click_v_track(&mut self, y: f64) {
        let s = self.vsb();
        if s.visible { self.sy = ((y-s.ty)/s.th * self.msy()).max(0.0); self.clamp(); }
    }

    // hit test
    pub fn hit_test(&mut self, cx: f64, cy: f64) -> String {
        self.ensure();
        let ox = self.origin_x(); let oy = self.origin_y();

        // scrollbars
        let hs = self.hsb();
        if hs.visible && cy >= hs.ty && cy <= hs.ty+hs.th && cx >= hs.tx && cx <= hs.tx+hs.tw {
            return hjson("h_scrollbar", None, None, None);
        }
        let vs = self.vsb();
        if vs.visible && cx >= vs.tx && cx <= vs.tx+vs.tw && cy >= vs.ty && cy <= vs.ty+vs.th {
            return hjson("v_scrollbar", None, None, None);
        }

        // brackets
        let bw = self.bdepth() as f64 * self.bs;
        if cx < bw && cy >= oy + self.frzh {
            let d = (cx / self.bs) as u32;
            let ly = cy - oy - self.frzh + self.sy;
            for (i, n) in self.rnodes.iter().enumerate() {
                if n.depth as u32 != d || n.children.is_empty() || n.is_subtotal || n.is_grand_total { continue; }
                if let Some(gr) = n.grid_index {
                    if gr < self.fr { continue; }
                    let sy = self.srow_px(gr);
                    let lr = self.last_desc(i).unwrap_or(gr);
                    let ey = self.srow_px(lr) + self.rowh(lr);
                    if ly >= sy && ly <= ey {
                        return hjson("row_bracket", Some(i as u32), None, Some(n.full_key.join("|")));
                    }
                }
            }
        }

        // cells
        if cx >= ox && cy >= oy {
            let rx = cx - ox; let ry = cy - oy;
            let col = if rx < self.frzw { px_to_frozen_col(rx, self.fc, &self.cw, self.dcw) }
                      else { px_to_scroll_col(rx - self.frzw + self.sx, self.fc, self.tc, &self.cw, self.dcw) };
            let row = if ry < self.frzh { px_to_frozen_row(ry, self.fr, &self.rh_map, self.drh) }
                      else { px_to_scroll_row(ry - self.frzh + self.sy, self.fr, self.tr, &self.rh_map, self.drh) };

            // col resize
            if ry < self.frzh {
                let ce = if rx < self.frzw {
                    self.fcol_px(col) + self.colw(col)
                } else {
                    self.scol_px(col) + self.colw(col) - self.sx + self.frzw
                };
                if (rx - ce).abs() < 5.0 {
                    return hjson("col_resize", None, Some(col), None);
                }
            }
            return hjson("cell", Some(row), Some(col), None);
        }
        hjson("none", None, None, None)
    }

    pub fn cell_screen_rect(&mut self, r: u32, c: u32) -> String {
        self.ensure();
        let ox = self.origin_x(); let oy = self.origin_y();
        let sx = if c < self.fc { self.fcol_px(c) + ox }
                 else { self.scol_px(c) - self.sx + ox + self.frzw };
        let sy = if r < self.fr { self.frow_px(r) + oy }
                 else { self.srow_px(r) - self.sy + oy + self.frzh };
        format!(r#"{{"x":{},"y":{},"w":{},"h":{}}}"#, sx, sy, self.colw(c), self.rowh(r))
    }

    pub fn render_frame(&mut self) -> String {
        self.ensure();
        let ox = self.origin_x(); let oy = self.origin_y();
        let fw = self.frzw; let fh = self.frzh;
        let dvw = self.dvw(); let dvh = self.dvh();

        let mut cells = Vec::new();
        let mut ch = Vec::new();
        let mut rh = Vec::new();

        // frozen cols
        let fcols: Vec<(u32,f64)> = {
            let mut v = Vec::new(); let mut px = 0.0;
            for c in 0..self.fc { v.push((c, ox+px)); px += self.colw(c); }
            v
        };
        // visible scrollable cols
        let scols: Vec<(u32,f64)> = {
            let mut v = Vec::new(); let mut px = 0.0_f64;
            for c in self.fc..self.tc {
                let w = self.colw(c); let s = px - self.sx;
                if s+w > 0.0 && s < dvw { v.push((c, ox+fw+s)); }
                px += w; if s > dvw { break; }
            }
            v
        };
        // frozen rows
        let frows: Vec<(u32,f64)> = {
            let mut v = Vec::new(); let mut py = 0.0;
            for r in 0..self.fr { v.push((r, oy+py)); py += self.rowh(r); }
            v
        };
        // visible scrollable rows
        let srows: Vec<(u32,f64)> = {
            let mut v = Vec::new(); let mut py = 0.0_f64;
            for r in self.fr..self.tr {
                let h = self.rowh(r); let s = py - self.sy;
                if s+h > 0.0 && s < dvh { v.push((r, oy+fh+s)); }
                py += h; if s > dvh { break; }
            }
            v
        };

        // emit 4 quadrants
        for &(r,sy) in frows.iter().chain(srows.iter()) {
            let h = self.rowh(r);
            for &(c,sx) in fcols.iter().chain(scols.iter()) {
                self.emit_cell(&mut cells, r, c, sx, sy, self.colw(c), h);
            }
        }

        // col headers
        for &(c,sx) in fcols.iter().chain(scols.iter()) {
            ch.push(VHeader { pos:sx, size:self.colw(c), label:col_label(c), index:c, highlighted: self.sel_c==c as i32 });
        }
        // row headers
        for &(r,sy) in &srows {
            rh.push(VHeader { pos:sy, size:self.rowh(r), label:(r+1).to_string(), index:r, highlighted: self.sel_r==r as i32 });
        }

        // brackets
        let rb = self.vis_brackets(oy, fh, dvh);

        let m = Metrics {
            ox, oy, fw, fh,
            fc: self.fc, fr: self.fr,
            rhw: self.rhw, chh: self.chh,
            bd: self.bdepth(), bs: self.bs, sbs: self.sbs,
        };

        let frame = Frame { cells, ch, rh, rb, m, hs: self.hsb(), vs: self.vsb() };
        serde_json::to_string(&frame).unwrap()
    }

    fn emit_cell(&self, cells: &mut Vec<VCell>, r: u32, c: u32, sx: f64, sy: f64, w: f64, h: f64) {
        let cd = self.cells.get(&(r,c));
        let sel = self.sel_r == r as i32 && self.sel_c == c as i32;
        let ed = self.ed_r == r as i32 && self.ed_c == c as i32;

        let (ct, ta, ind) = if let Some(d) = cd {
            let ct = if d.is_grand_total { 4u8 } else if d.is_subtotal { 3 }
                     else if d.is_header { 1 } else if d.is_value { 2 } else { 0 };
            let ta = if d.is_value { 2u8 }
                     else if d.is_header && c < self.fc { 0 }
                     else if d.is_header { 1 } else { 0 };
            let ind = if c < self.fc && !d.is_subtotal && !d.is_grand_total && r >= self.fr {
                self.rnodes.iter().find(|n| n.grid_index == Some(r)).map(|n| n.depth as f64 * 12.0).unwrap_or(0.0)
            } else { 0.0 };
            (ct, ta, ind)
        } else { (0u8, 0u8, 0.0) };

        let bg = if ed { "#FFFFFF".into() } else if sel { "#1E3A5F".into() }
                 else { cd.and_then(|d| d.bg.clone()).unwrap_or("#0F172A".into()) };
        let fg = cd.and_then(|d| d.fg.clone()).unwrap_or("#E2E8F0".into());

        cells.push(VCell {
            sx,sy,w,h,
            text: cd.map(|d| d.text.clone()).unwrap_or_default(),
            bg, fg, bold: cd.map(|d| d.bold).unwrap_or(false),
            row:r, col:c, selected:sel, editing:ed,
            cell_type:ct, text_align:ta, indent:ind,
        });
    }

    fn vis_brackets(&self, oy: f64, fh: f64, dvh: f64) -> Vec<VBracket> {
        let mut out = Vec::new();
        for (i,n) in self.rnodes.iter().enumerate() {
            if n.children.is_empty() || n.is_subtotal || n.is_grand_total { continue; }
            if let Some(gr) = n.grid_index {
                if gr < self.fr { continue; }
                let lr = self.last_desc(i).unwrap_or(gr);
                let sp = self.srow_px(gr) - self.sy + oy + fh;
                let ep = self.srow_px(lr) + self.rowh(lr) - self.sy + oy + fh;
                if ep > oy + fh && sp < oy + fh + dvh {
                    out.push(VBracket { id:i as u32, label:n.key.clone(), start:sp, end:ep, depth:n.depth as u32, collapsed:n.collapsed });
                }
            }
        }
        out
    }

    pub fn cell_count(&self) -> u32 { self.cells.len() as u32 }
    pub fn get_total_rows(&self) -> u32 { self.tr }
    pub fn get_total_cols(&self) -> u32 { self.tc }
}

// ─── Free functions ──────────────────────────────────────────

fn build_tree(
    recs: &[RawRecord], fields: &[PivotField], nodes: &mut Vec<PivotNode>,
    subtotals: bool, grand: bool, collapsed: &BTreeSet<String>,
) -> Vec<usize> {
    if fields.is_empty() {
        nodes.push(PivotNode {
            key: String::new(), full_key: vec![], depth: 0,
            children: vec![], is_subtotal: false, is_grand_total: false,
            collapsed: false, grid_index: None,
        });
        let mut roots = vec![0];
        if grand {
            let gi = nodes.len();
            nodes.push(PivotNode {
                key: "Grand Total".into(), full_key: vec!["__gt__".into()],
                depth: 0, children: vec![], is_subtotal: false,
                is_grand_total: true, collapsed: false, grid_index: None,
            });
            roots.push(gi);
        }
        return roots;
    }

    let mut roots = tree_level(recs, fields, nodes, subtotals, collapsed, 0, &[], &[]);
    if grand {
        let gi = nodes.len();
        nodes.push(PivotNode {
            key: "Grand Total".into(), full_key: vec!["__gt__".into()],
            depth: 0, children: vec![], is_subtotal: false,
            is_grand_total: true, collapsed: false, grid_index: None,
        });
        roots.push(gi);
    }
    roots
}

fn tree_level(
    recs: &[RawRecord], fields: &[PivotField], nodes: &mut Vec<PivotNode>,
    subtotals: bool, collapsed: &BTreeSet<String>,
    depth: usize, pkey: &[String], filter: &[(usize, String)],
) -> Vec<usize> {
    if depth >= fields.len() { return vec![]; }
    let fname = &fields[depth].name;

    let mut uniq: BTreeSet<String> = BTreeSet::new();
    'outer: for r in recs {
        for &(fi, ref fv) in filter {
            if r.strings.get(&fields[fi].name).map(|s| s.as_str()).unwrap_or("") != fv.as_str() { continue 'outer; }
        }
        if let Some(v) = r.strings.get(fname) { uniq.insert(v.clone()); }
    }

    let mut out = Vec::new();
    for val in &uniq {
        let mut fk = pkey.to_vec(); fk.push(val.clone());
        let ks = fk.join("|");
        let is_c = collapsed.contains(&ks);
        let ni = nodes.len();
        nodes.push(PivotNode {
            key: val.clone(), full_key: fk.clone(), depth,
            children: vec![], is_subtotal: false, is_grand_total: false,
            collapsed: is_c, grid_index: None,
        });
        let mut nf = filter.to_vec(); nf.push((depth, val.clone()));
        let ch = tree_level(recs, fields, nodes, subtotals, collapsed, depth+1, &fk, &nf);
        nodes[ni].children = ch;
        out.push(ni);
        if subtotals && depth < fields.len()-1 && !nodes[ni].children.is_empty() {
            let si = nodes.len();
            let mut sk = fk; sk.push("__sub__".into());
            nodes.push(PivotNode {
                key: format!("{} Total", val), full_key: sk, depth,
                children: vec![], is_subtotal: true, is_grand_total: false,
                collapsed: false, grid_index: None,
            });
            out.push(si);
        }
    }
    out
}

fn assign_rows(nodes: &mut Vec<PivotNode>, ids: &[usize], next: &mut u32) {
    for &i in ids {
        nodes[i].grid_index = Some(*next); *next += 1;
        if !nodes[i].collapsed && !nodes[i].children.is_empty() {
            let ch = nodes[i].children.clone();
            assign_rows(nodes, &ch, next);
        }
    }
}

fn assign_cols(nodes: &mut Vec<PivotNode>, ids: &[usize], nv: u32, next: &mut u32) {
    for &i in ids {
        nodes[i].grid_index = Some(*next);
        if nodes[i].children.is_empty() || nodes[i].collapsed {
            *next += nv;
        } else {
            let ch = nodes[i].children.clone();
            assign_cols(nodes, &ch, nv, next);
        }
    }
}

fn write_headers(cfg: &PivotConfig, fr: u32, cells: &mut HashMap<(u32,u32), CellData>) {
    let vr = fr.saturating_sub(1);
    for (i,f) in cfg.row_fields.iter().enumerate() {
        cells.insert((vr, i as u32), CellData {
            text: f.name.clone(), bold: true, is_header: true,
            bg: Some("#1a2744".into()), fg: Some("#93C5FD".into()), ..Default::default()
        });
    }
    for (i,f) in cfg.col_fields.iter().enumerate() {
        cells.insert((i as u32, 0), CellData {
            text: f.name.clone(), bold: true, is_header: true,
            bg: Some("#1a2744".into()), fg: Some("#93C5FD".into()), ..Default::default()
        });
    }
}

fn write_row_labels(nodes: &[PivotNode], fc: u32, fr: u32, cells: &mut HashMap<(u32,u32), CellData>) {
    for n in nodes {
        if let Some(gr) = n.grid_index {
            let tc = if n.is_subtotal || n.is_grand_total { 0 }
                     else { (n.depth as u32).min(fc.saturating_sub(1)) };
            cells.insert((gr, tc), CellData {
                text: n.key.clone(),
                bold: n.is_subtotal || n.is_grand_total || n.depth == 0,
                is_header: true, is_subtotal: n.is_subtotal,
                is_grand_total: n.is_grand_total, ..Default::default()
            });
        }
    }
}

fn write_col_labels(nodes: &[PivotNode], cfg: &PivotConfig, fr: u32, cells: &mut HashMap<(u32,u32), CellData>) {
    let nv = cfg.value_fields.len().max(1) as u32;
    for n in nodes {
        if let Some(gc) = n.grid_index {
            cells.insert((n.depth as u32, gc), CellData {
                text: n.key.clone(), bold: true, is_header: true,
                is_subtotal: n.is_subtotal, is_grand_total: n.is_grand_total, ..Default::default()
            });
            let leaf = n.children.is_empty() || n.collapsed;
            if leaf && cfg.value_fields.len() > 1 {
                let vlr = fr.saturating_sub(1);
                for (vi,vf) in cfg.value_fields.iter().enumerate() {
                    cells.insert((vlr, gc + vi as u32), CellData {
                        text: vf.display_label(), bold: true, is_header: true, ..Default::default()
                    });
                }
            }
        }
    }
}

/// Build a pre-indexed aggregation map.
/// Key: (row_key, col_key) where keys are the joined field values for matching.
/// For subtotals/grand totals, we use partial keys.
fn build_agg_index(records: &[RawRecord], cfg: &PivotConfig) -> AggIndex {
    let nv = cfg.value_fields.len();
    let mut idx: AggIndex = HashMap::new();

    // For each record, generate all (row_key_prefix, col_key_prefix) combinations
    // that this record contributes to. This covers subtotals and grand totals.
    for rec in records {
        let row_vals: Vec<String> = cfg.row_fields.iter()
            .map(|f| rec.strings.get(&f.name).cloned().unwrap_or_default())
            .collect();
        let col_vals: Vec<String> = cfg.col_fields.iter()
            .map(|f| rec.strings.get(&f.name).cloned().unwrap_or_default())
            .collect();

        let vals: Vec<f64> = cfg.value_fields.iter()
            .map(|vf| rec.numbers.get(&vf.name).copied().unwrap_or(0.0))
            .collect();

        // Generate all row key prefixes (for subtotals)
        // Full key: "A|B|C", partial: "A|B", "A", "" (grand total)
        let mut row_keys: Vec<String> = Vec::new();
        row_keys.push("__gt__".into()); // grand total
        for i in 0..row_vals.len() {
            row_keys.push(row_vals[..=i].join("|"));
        }
        // subtotal keys
        for i in 0..row_vals.len() {
            let mut k = row_vals[..=i].join("|");
            k.push_str("|__sub__");
            row_keys.push(k);
        }

        let mut col_keys: Vec<String> = Vec::new();
        col_keys.push("__gt__".into());
        for i in 0..col_vals.len() {
            col_keys.push(col_vals[..=i].join("|"));
        }
        for i in 0..col_vals.len() {
            let mut k = col_vals[..=i].join("|");
            k.push_str("|__sub__");
            col_keys.push(k);
        }
        // empty key for no-field case
        if cfg.row_fields.is_empty() { row_keys.push(String::new()); }
        if cfg.col_fields.is_empty() { col_keys.push(String::new()); }

        for rk in &row_keys {
            for ck in &col_keys {
                let entry = idx.entry((rk.clone(), ck.clone())).or_insert_with(|| vec![Vec::new(); nv]);
                for (vi, &v) in vals.iter().enumerate() {
                    entry[vi].push(v);
                }
            }
        }
    }

    idx
}

fn node_agg_key(node: &PivotNode) -> String {
    if node.is_grand_total { return "__gt__".into(); }
    if node.full_key.is_empty() { return String::new(); }
    node.full_key.join("|")
}

fn write_data_indexed(
    rnodes: &[PivotNode], cnodes: &[PivotNode], croots: &[usize],
    cfg: &PivotConfig, agg: &AggIndex, cells: &mut HashMap<(u32,u32), CellData>,
) {
    let leaf_cols = collect_leaves(cnodes, croots);
    let nv = cfg.value_fields.len();

    for rn in rnodes {
        let gr = match rn.grid_index { Some(r) => r, None => continue };
        let rk = node_agg_key(rn);

        for lc in &leaf_cols {
            let gc = match lc.grid_index { Some(c) => c, None => continue };
            let ck = node_agg_key(lc);

            if let Some(buckets) = agg.get(&(rk.clone(), ck.clone())) {
                for (vi, vf) in cfg.value_fields.iter().enumerate() {
                    if vi < buckets.len() && !buckets[vi].is_empty() {
                        let result = vf.agg.apply(&buckets[vi]);
                        cells.insert((gr, gc + vi as u32), CellData {
                            text: fmt_num(result), is_value: true,
                            is_subtotal: rn.is_subtotal || lc.is_subtotal,
                            is_grand_total: rn.is_grand_total || lc.is_grand_total,
                            bold: rn.is_subtotal || rn.is_grand_total || lc.is_subtotal || lc.is_grand_total,
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }
}

fn collect_leaves(nodes: &[PivotNode], roots: &[usize]) -> Vec<PivotNode> {
    let mut out = Vec::new();
    fn walk(nodes: &[PivotNode], ids: &[usize], out: &mut Vec<PivotNode>) {
        for &i in ids {
            if nodes[i].children.is_empty() || nodes[i].collapsed {
                out.push(nodes[i].clone());
            } else { walk(nodes, &nodes[i].children, out); }
        }
    }
    walk(nodes, roots, &mut out);
    out
}

fn style_cells(cells: &mut HashMap<(u32,u32), CellData>, fr: u32, fc: u32) {
    let keys: Vec<(u32,u32)> = cells.keys().cloned().collect();
    for (r,c) in keys {
        let d = cells.get(&(r,c)).unwrap().clone();
        let mut s = d;
        if s.is_grand_total { s.bg = Some("#1a3350".into()); s.fg = Some("#FBBF24".into()); s.bold = true; }
        else if s.is_subtotal { s.bg = Some("#172544".into()); s.fg = Some("#93C5FD".into()); s.bold = true; }
        else if s.is_header && r < fr { s.bg = Some("#1a2744".into()); s.fg = Some("#94A3B8".into()); }
        else if s.is_header { s.bg = Some("#152238".into()); s.fg = Some("#CBD5E1".into()); }
        else if s.is_value {
            let dr = r.saturating_sub(fr);
            s.bg = Some(if dr%2==0 { "#0F172A".into() } else { "#131d30".into() });
            s.fg = Some("#E2E8F0".into());
        }
        cells.insert((r,c), s);
    }
}

fn init_col_widths(cw: &mut BTreeMap<u32,f64>, fc: u32, tc: u32, dcw: f64) {
    for c in 0..fc { cw.entry(c).or_insert(130.0); }
    for c in fc..tc { cw.entry(c).or_insert(dcw); }
}

fn px_to_frozen_col(x: f64, fc: u32, cw: &BTreeMap<u32,f64>, dcw: f64) -> u32 {
    let mut a = 0.0;
    for c in 0..fc { let w = *cw.get(&c).unwrap_or(&dcw); if a+w > x { return c; } a += w; }
    fc.saturating_sub(1)
}
fn px_to_frozen_row(y: f64, fr: u32, rh: &BTreeMap<u32,f64>, drh: f64) -> u32 {
    let mut a = 0.0;
    for r in 0..fr { let h = *rh.get(&r).unwrap_or(&drh); if a+h > y { return r; } a += h; }
    fr.saturating_sub(1)
}
fn px_to_scroll_col(x: f64, fc: u32, tc: u32, cw: &BTreeMap<u32,f64>, dcw: f64) -> u32 {
    let mut a = 0.0;
    for c in fc..tc { let w = *cw.get(&c).unwrap_or(&dcw); if a+w > x { return c; } a += w; }
    tc.saturating_sub(1)
}
fn px_to_scroll_row(y: f64, fr: u32, tr: u32, rh: &BTreeMap<u32,f64>, drh: f64) -> u32 {
    let mut a = 0.0;
    for r in fr..tr { let h = *rh.get(&r).unwrap_or(&drh); if a+h > y { return r; } a += h; }
    tr.saturating_sub(1)
}

fn hjson(t: &str, r: Option<u32>, c: Option<u32>, k: Option<String>) -> String {
    serde_json::to_string(&Hit { t: t.into(), r, c, k }).unwrap()
}

fn col_label(mut c: u32) -> String {
    let mut s = String::new();
    loop { s.insert(0, (b'A'+(c%26) as u8) as char); if c < 26 { break; } c = c/26-1; }
    s
}

fn fmt_num(v: f64) -> String {
    if v == v.floor() && v.abs() < 1e15 {
        let i = v as i64; let neg = i < 0;
        let s = i.unsigned_abs().to_string();
        let b: Vec<u8> = s.bytes().collect();
        let mut r = Vec::new();
        for (j,&ch) in b.iter().enumerate() {
            if j > 0 && (b.len()-j)%3==0 { r.push(b','); }
            r.push(ch);
        }
        let f = String::from_utf8(r).unwrap();
        if neg { format!("-{}",f) } else { f }
    } else { format!("{:.1}",v) }
}
