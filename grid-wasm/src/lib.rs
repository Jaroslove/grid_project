use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, BTreeMap};

// ─── Cell ────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CellData {
    pub text: String,
    pub bg_color: Option<String>,
    pub fg_color: Option<String>,
    pub bold: bool,
}

impl Default for CellData {
    fn default() -> Self {
        Self {
            text: String::new(),
            bg_color: None,
            fg_color: None,
            bold: false,
        }
    }
}

// Simple pivot input record: one row key, one column key, and a numeric value.
#[derive(Deserialize, Debug)]
struct PivotRecord {
    row: String,
    col: String,
    value: f64,
}

// ─── Group ───────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Group {
    pub id: u32,
    pub label: String,
    pub members: Vec<u32>,
    pub collapsed: bool,
    pub depth: u32,
    pub parent: Option<u32>,
}

// ─── Render output types ─────────────────────────────────────────────

#[derive(Serialize, Debug)]
pub struct VisibleCell {
    pub sx: f64,
    pub sy: f64,
    pub w: f64,
    pub h: f64,
    pub text: String,
    pub bg: String,
    pub fg: String,
    pub bold: bool,
    pub row: u32,
    pub col: u32,
    pub selected: bool,
    pub editing: bool,
}

#[derive(Serialize, Debug)]
pub struct VisibleHeader {
    pub pos: f64,
    pub size: f64,
    pub label: String,
    pub index: u32,
    pub highlighted: bool,
}

#[derive(Serialize, Debug)]
pub struct GroupBracket {
    pub id: u32,
    pub label: String,
    pub start: f64,
    pub end: f64,
    pub depth: u32,
    pub collapsed: bool,
    pub is_row: bool,
}

#[derive(Serialize, Debug)]
pub struct GridMetrics {
    pub row_header_width: f64,
    pub col_header_height: f64,
    pub group_cols_depth: u32,
    pub group_rows_depth: u32,
    pub bracket_size: f64,
    pub content_origin_x: f64,
    pub content_origin_y: f64,
}

#[derive(Serialize, Debug)]
pub struct RenderFrame {
    pub cells: Vec<VisibleCell>,
    pub col_headers: Vec<VisibleHeader>,
    pub row_headers: Vec<VisibleHeader>,
    pub row_brackets: Vec<GroupBracket>,
    pub col_brackets: Vec<GroupBracket>,
    pub metrics: GridMetrics,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HitResult {
    #[serde(rename = "type")]
    pub hit_type: String,
    pub row: Option<u32>,
    pub col: Option<u32>,
    pub group_id: Option<u32>,
}

// ─── Grid State ──────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct Grid {
    cells: HashMap<(u32, u32), CellData>,
    col_widths: BTreeMap<u32, f64>,
    row_heights: BTreeMap<u32, f64>,
    default_col_width: f64,
    default_row_height: f64,

    scroll_x: f64,
    scroll_y: f64,
    viewport_w: f64,
    viewport_h: f64,

    row_groups: Vec<Group>,
    col_groups: Vec<Group>,
    next_group_id: u32,

    hidden_rows: Vec<bool>,
    hidden_cols: Vec<bool>,
    hidden_dirty: bool,

    sel_row: i32,
    sel_col: i32,
    edit_row: i32,
    edit_col: i32,

    row_header_width: f64,
    col_header_height: f64,
    bracket_size: f64,

    // Column resize tracking
    resize_col: i32,
    resize_start_x: f64,
    resize_start_width: f64,
}

#[wasm_bindgen]
impl Grid {
    #[wasm_bindgen(constructor)]
    pub fn new(vw: f64, vh: f64) -> Self {
        Self {
            cells: HashMap::new(),
            col_widths: BTreeMap::new(),
            row_heights: BTreeMap::new(),
            default_col_width: 120.0,
            default_row_height: 30.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
            viewport_w: vw,
            viewport_h: vh,
            row_groups: Vec::new(),
            col_groups: Vec::new(),
            next_group_id: 1,
            hidden_rows: Vec::new(),
            hidden_cols: Vec::new(),
            hidden_dirty: true,
            sel_row: -1,
            sel_col: -1,
            edit_row: -1,
            edit_col: -1,
            row_header_width: 60.0,
            col_header_height: 28.0,
            bracket_size: 20.0,
            resize_col: -1,
            resize_start_x: 0.0,
            resize_start_width: 0.0,
        }
    }

    // ─── Cell ops ────────────────────────────────────────────────

    pub fn set_cell(&mut self, row: u32, col: u32, text: &str) {
        let test = "test".to_string();
        let value = format!("{}_{}", test, text);
        self.cells.entry((row, col)).or_default().text = text.to_string();
    }

    pub fn set_cell_style(&mut self, row: u32, col: u32, bg: &str, fg: &str, bold: bool) {
        let entry = self.cells.entry((row, col)).or_default();
        if !bg.is_empty() { entry.bg_color = Some(bg.to_string()); }
        if !fg.is_empty() { entry.fg_color = Some(fg.to_string()); }
        entry.bold = bold;
    }

    pub fn get_cell_text(&self, row: u32, col: u32) -> String {
        self.cells.get(&(row, col)).map(|c| c.text.clone()).unwrap_or_default()
    }

    pub fn clear_cell(&mut self, row: u32, col: u32) {
        self.cells.remove(&(row, col));
    }

    pub fn load_cells_json(&mut self, json: &str) {
        if let Ok(data) = serde_json::from_str::<Vec<(u32, u32, String)>>(json) {
            for (r, c, t) in data {
                self.set_cell(r, c, &t);
            }
        }
    }

    /// Load data in a simple pivot-table shape.
    ///
    /// Expects JSON like:
    ///   [{ "row": "Row A", "col": "Col 1", "value": 10.0 }, ...]
    ///
    /// It will:
    /// - Clear existing cells and groups.
    /// - Use row index 0 as header row (column labels).
    /// - Use column index 0 as header column (row labels).
    /// - Fill numeric cells with the sum of `value` for each (row, col) pair.
    pub fn load_pivot_json(&mut self, json: &str) {
        let records: Vec<PivotRecord> = match serde_json::from_str(json) {
            Ok(v) => v,
            Err(_) => return,
        };

        // Reset grid content but keep sizing/viewport.
        self.cells.clear();
        self.row_groups.clear();
        self.col_groups.clear();
        self.hidden_rows.clear();
        self.hidden_cols.clear();
        self.hidden_dirty = true;
        self.sel_row = -1;
        self.sel_col = -1;
        self.edit_row = -1;
        self.edit_col = -1;

        // Map row/col labels to indices. Reserve 0 for headers.
        let mut row_index: BTreeMap<String, u32> = BTreeMap::new();
        let mut col_index: BTreeMap<String, u32> = BTreeMap::new();
        let mut next_row: u32 = 1;
        let mut next_col: u32 = 1;

        let mut agg: HashMap<(u32, u32), f64> = HashMap::new();

        for rec in records {
            let r = *row_index.entry(rec.row).or_insert_with(|| {
                let idx = next_row;
                next_row += 1;
                idx
            });
            let c = *col_index.entry(rec.col).or_insert_with(|| {
                let idx = next_col;
                next_col += 1;
                idx
            });

            *agg.entry((r, c)).or_insert(0.0) += rec.value;
        }

        // Header row: column labels in row 0, columns 1..N
        for (label, c) in &col_index {
            self.set_cell(0, *c, label);
        }

        // Header column: row labels in column 0, rows 1..M
        for (label, r) in &row_index {
            self.set_cell(*r, 0, label);
        }

        // Data cells: aggregated values
        for ((r, c), val) in agg {
            self.set_cell(r, c, &format!("{}", val));
        }
    }

    // ─── Sizing ──────────────────────────────────────────────────

    pub fn set_col_width(&mut self, col: u32, w: f64) {
        self.col_widths.insert(col, w.max(30.0));
    }

    pub fn set_row_height(&mut self, row: u32, h: f64) {
        self.row_heights.insert(row, h.max(14.0));
    }

    pub fn set_default_col_width(&mut self, w: f64) { self.default_col_width = w.max(30.0); }
    pub fn set_default_row_height(&mut self, h: f64) { self.default_row_height = h.max(14.0); }

    fn col_w(&self, c: u32) -> f64 {
        *self.col_widths.get(&c).unwrap_or(&self.default_col_width)
    }
    fn row_h(&self, r: u32) -> f64 {
        *self.row_heights.get(&r).unwrap_or(&self.default_row_height)
    }

    // ─── Scroll / viewport ───────────────────────────────────────

    pub fn set_viewport(&mut self, w: f64, h: f64) {
        self.viewport_w = w;
        self.viewport_h = h;
    }

    pub fn scroll_by(&mut self, dx: f64, dy: f64) {
        self.scroll_x = (self.scroll_x + dx).max(0.0);
        self.scroll_y = (self.scroll_y + dy).max(0.0);
    }

    pub fn set_scroll(&mut self, x: f64, y: f64) {
        self.scroll_x = x.max(0.0);
        self.scroll_y = y.max(0.0);
    }

    pub fn get_scroll_x(&self) -> f64 { self.scroll_x }
    pub fn get_scroll_y(&self) -> f64 { self.scroll_y }

    // ─── Selection / editing ─────────────────────────────────────

    pub fn select(&mut self, r: i32, c: i32) { self.sel_row = r; self.sel_col = c; }
    pub fn sel_row(&self) -> i32 { self.sel_row }
    pub fn sel_col(&self) -> i32 { self.sel_col }

    pub fn edit(&mut self, r: i32, c: i32) { self.edit_row = r; self.edit_col = c; }
    pub fn edit_row(&self) -> i32 { self.edit_row }
    pub fn edit_col(&self) -> i32 { self.edit_col }

    pub fn move_selection(&mut self, dr: i32, dc: i32) {
        if self.sel_row < 0 { self.sel_row = 0; }
        if self.sel_col < 0 { self.sel_col = 0; }
        self.sel_row = (self.sel_row + dr).max(0);
        self.sel_col = (self.sel_col + dc).max(0);

        // Skip hidden rows/cols
        self.rebuild_hidden();
        while self.is_row_hidden(self.sel_row as u32) && self.sel_row >= 0 {
            self.sel_row += if dr >= 0 { 1 } else { -1 };
        }
        while self.is_col_hidden(self.sel_col as u32) && self.sel_col >= 0 {
            self.sel_col += if dc >= 0 { 1 } else { -1 };
        }
        self.sel_row = self.sel_row.max(0);
        self.sel_col = self.sel_col.max(0);

        self.ensure_visible(self.sel_row as u32, self.sel_col as u32);
    }

    /// Scroll so that the given cell is visible
    fn ensure_visible(&mut self, row: u32, col: u32) {
        let metrics = self.metrics();
        let ox = metrics.content_origin_x;
        let oy = metrics.content_origin_y;
        let cw = self.viewport_w - ox;
        let ch = self.viewport_h - oy;

        // Column
        let col_start = self.col_pixel(col);
        let col_end = col_start + self.col_w(col);
        if col_start < self.scroll_x {
            self.scroll_x = col_start;
        } else if col_end > self.scroll_x + cw {
            self.scroll_x = col_end - cw;
        }

        // Row
        let row_start = self.row_pixel(row);
        let row_end = row_start + self.row_h(row);
        if row_start < self.scroll_y {
            self.scroll_y = row_start;
        } else if row_end > self.scroll_y + ch {
            self.scroll_y = row_end - ch;
        }
    }

    // ─── Column resize ──────────────────────────────────────────

    pub fn start_col_resize(&mut self, col: i32, start_x: f64) {
        if col >= 0 {
            self.resize_col = col;
            self.resize_start_x = start_x;
            self.resize_start_width = self.col_w(col as u32);
        }
    }

    pub fn update_col_resize(&mut self, current_x: f64) {
        if self.resize_col >= 0 {
            let delta = current_x - self.resize_start_x;
            let new_width = (self.resize_start_width + delta).max(30.0);
            self.col_widths.insert(self.resize_col as u32, new_width);
        }
    }

    pub fn end_col_resize(&mut self) {
        self.resize_col = -1;
    }

    pub fn is_resizing(&self) -> bool {
        self.resize_col >= 0
    }

    // ─── Groups ──────────────────────────────────────────────────

    pub fn add_row_group(&mut self, label: &str, members_json: &str, parent_id: i32) -> u32 {
        let members: Vec<u32> = serde_json::from_str(members_json).unwrap_or_default();
        let parent = if parent_id > 0 { Some(parent_id as u32) } else { None };
        let depth = parent
            .and_then(|pid| self.row_groups.iter().find(|g| g.id == pid))
            .map(|g| g.depth + 1)
            .unwrap_or(0);

        let id = self.next_group_id;
        self.next_group_id += 1;
        self.row_groups.push(Group { id, label: label.into(), members, collapsed: false, depth, parent });
        self.hidden_dirty = true;
        id
    }

    pub fn add_col_group(&mut self, label: &str, members_json: &str, parent_id: i32) -> u32 {
        let members: Vec<u32> = serde_json::from_str(members_json).unwrap_or_default();
        let parent = if parent_id > 0 { Some(parent_id as u32) } else { None };
        let depth = parent
            .and_then(|pid| self.col_groups.iter().find(|g| g.id == pid))
            .map(|g| g.depth + 1)
            .unwrap_or(0);

        let id = self.next_group_id;
        self.next_group_id += 1;
        self.col_groups.push(Group { id, label: label.into(), members, collapsed: false, depth, parent });
        self.hidden_dirty = true;
        id
    }

    pub fn toggle_group(&mut self, group_id: u32) {
        for g in self.row_groups.iter_mut().chain(self.col_groups.iter_mut()) {
            if g.id == group_id {
                g.collapsed = !g.collapsed;
                self.hidden_dirty = true;
                return;
            }
        }
    }

    pub fn remove_group(&mut self, group_id: u32) {
        self.row_groups.retain(|g| g.id != group_id);
        self.col_groups.retain(|g| g.id != group_id);
        // Re-parent children
        let orphan_ids: Vec<u32> = self.row_groups.iter()
            .chain(self.col_groups.iter())
            .filter(|g| g.parent == Some(group_id))
            .map(|g| g.id)
            .collect();
        for g in self.row_groups.iter_mut().chain(self.col_groups.iter_mut()) {
            if orphan_ids.contains(&g.id) {
                g.parent = None;
                g.depth = 0;
            }
        }
        self.hidden_dirty = true;
    }

    // ─── Hidden rebuild ──────────────────────────────────────────

    fn rebuild_hidden(&mut self) {
        if !self.hidden_dirty { return; }

        let max_r = self.row_groups.iter().flat_map(|g| &g.members).copied().max().unwrap_or(0) as usize;
        let max_c = self.col_groups.iter().flat_map(|g| &g.members).copied().max().unwrap_or(0) as usize;

        self.hidden_rows = vec![false; max_r + 1];
        self.hidden_cols = vec![false; max_c + 1];

        let rg = self.row_groups.clone();
        let cg = self.col_groups.clone();

        let hidden_r_ids = self.collapsed_ids(&rg);
        let hidden_c_ids = self.collapsed_ids(&cg);

        for g in &rg {
            if hidden_r_ids.contains(&g.id) {
                for &m in &g.members {
                    if (m as usize) < self.hidden_rows.len() { self.hidden_rows[m as usize] = true; }
                }
            }
        }
        for g in &cg {
            if hidden_c_ids.contains(&g.id) {
                for &m in &g.members {
                    if (m as usize) < self.hidden_cols.len() { self.hidden_cols[m as usize] = true; }
                }
            }
        }

        self.hidden_dirty = false;
    }

    fn collapsed_ids(&self, groups: &[Group]) -> Vec<u32> {
        let mut out = Vec::new();
        for g in groups {
            if g.collapsed || self.ancestor_collapsed(g.id, groups) {
                out.push(g.id);
                self.descendants(g.id, groups, &mut out);
            }
        }
        out.sort();
        out.dedup();
        out
    }

    fn descendants(&self, pid: u32, groups: &[Group], out: &mut Vec<u32>) {
        for g in groups {
            if g.parent == Some(pid) && !out.contains(&g.id) {
                out.push(g.id);
                self.descendants(g.id, groups, out);
            }
        }
    }

    fn ancestor_collapsed(&self, gid: u32, groups: &[Group]) -> bool {
        let g = match groups.iter().find(|g| g.id == gid) { Some(g) => g, None => return false };
        match g.parent {
            Some(pid) => {
                groups.iter().find(|g| g.id == pid)
                    .map(|p| p.collapsed || self.ancestor_collapsed(pid, groups))
                    .unwrap_or(false)
            },
            None => false,
        }
    }

    fn is_row_hidden(&self, r: u32) -> bool {
        (r as usize) < self.hidden_rows.len() && self.hidden_rows[r as usize]
    }
    fn is_col_hidden(&self, c: u32) -> bool {
        (c as usize) < self.hidden_cols.len() && self.hidden_cols[c as usize]
    }

    fn max_row_depth(&self) -> u32 { self.row_groups.iter().map(|g| g.depth + 1).max().unwrap_or(0) }
    fn max_col_depth(&self) -> u32 { self.col_groups.iter().map(|g| g.depth + 1).max().unwrap_or(0) }

    // ─── Pixel math ──────────────────────────────────────────────

    fn col_pixel(&self, target: u32) -> f64 {
        let mut acc = 0.0;
        for c in 0..target {
            if !self.is_col_hidden(c) { acc += self.col_w(c); }
        }
        acc
    }

    fn row_pixel(&self, target: u32) -> f64 {
        let mut acc = 0.0;
        for r in 0..target {
            if !self.is_row_hidden(r) { acc += self.row_h(r); }
        }
        acc
    }

    fn pixel_to_col(&self, x: f64) -> u32 {
        let mut acc = 0.0;
        let mut c: u32 = 0;
        loop {
            if !self.is_col_hidden(c) {
                let w = self.col_w(c);
                if acc + w > x { return c; }
                acc += w;
            }
            c += 1;
            if c > 999_999 { return c; }
        }
    }

    fn pixel_to_row(&self, y: f64) -> u32 {
        let mut acc = 0.0;
        let mut r: u32 = 0;
        loop {
            if !self.is_row_hidden(r) {
                let h = self.row_h(r);
                if acc + h > y { return r; }
                acc += h;
            }
            r += 1;
            if r > 999_999 { return r; }
        }
    }

    fn group_pixel_range_col(&self, g: &Group) -> (f64, f64) {
        if g.members.is_empty() { return (0.0, 0.0); }
        let lo = *g.members.iter().min().unwrap();
        let hi = *g.members.iter().max().unwrap();
        (self.col_pixel(lo), self.col_pixel(hi) + self.col_w(hi))
    }

    fn group_pixel_range_row(&self, g: &Group) -> (f64, f64) {
        if g.members.is_empty() { return (0.0, 0.0); }
        let lo = *g.members.iter().min().unwrap();
        let hi = *g.members.iter().max().unwrap();
        (self.row_pixel(lo), self.row_pixel(hi) + self.row_h(hi))
    }

    fn metrics(&self) -> GridMetrics {
        let rd = self.max_row_depth();
        let cd = self.max_col_depth();
        let ox = rd as f64 * self.bracket_size + self.row_header_width;
        let oy = cd as f64 * self.bracket_size + self.col_header_height;
        GridMetrics {
            row_header_width: self.row_header_width,
            col_header_height: self.col_header_height,
            group_rows_depth: rd,
            group_cols_depth: cd,
            bracket_size: self.bracket_size,
            content_origin_x: ox,
            content_origin_y: oy,
        }
    }

    // ─── Hit test ────────────────────────────────────────────────

    pub fn hit_test(&mut self, cx: f64, cy: f64) -> String {
        self.rebuild_hidden();
        let m = self.metrics();

        // Column group area
        if cy < m.group_cols_depth as f64 * m.bracket_size && cx >= m.content_origin_x {
            let depth = (cy / m.bracket_size) as u32;
            let data_x = cx - m.content_origin_x + self.scroll_x;
            for g in &self.col_groups {
                if g.depth != depth { continue; }
                let (s, e) = self.group_pixel_range_col(g);
                if data_x >= s && data_x <= e {
                    let r = HitResult { hit_type: "col_group".into(), row: None, col: None, group_id: Some(g.id) };
                    return serde_json::to_string(&r).unwrap();
                }
            }
        }

        // Row group area
        if cx < m.group_rows_depth as f64 * m.bracket_size && cy >= m.content_origin_y {
            let depth = (cx / m.bracket_size) as u32;
            let data_y = cy - m.content_origin_y + self.scroll_y;
            for g in &self.row_groups {
                if g.depth != depth { continue; }
                let (s, e) = self.group_pixel_range_row(g);
                if data_y >= s && data_y <= e {
                    let r = HitResult { hit_type: "row_group".into(), row: None, col: None, group_id: Some(g.id) };
                    return serde_json::to_string(&r).unwrap();
                }
            }
        }

        // Column header — check for resize handle
        if cy >= m.content_origin_y - m.col_header_height && cy < m.content_origin_y && cx >= m.content_origin_x {
            let data_x = cx - m.content_origin_x + self.scroll_x;
            let col = self.pixel_to_col(data_x);
            let col_end = self.col_pixel(col) + self.col_w(col);
            let dist_to_edge = (data_x - col_end).abs();
            if dist_to_edge < 5.0 {
                let r = HitResult { hit_type: "col_resize".into(), row: None, col: Some(col), group_id: None };
                return serde_json::to_string(&r).unwrap();
            }
            let r = HitResult { hit_type: "col_header".into(), row: None, col: Some(col), group_id: None };
            return serde_json::to_string(&r).unwrap();
        }

        // Cell
        if cx >= m.content_origin_x && cy >= m.content_origin_y {
            let data_x = cx - m.content_origin_x + self.scroll_x;
            let data_y = cy - m.content_origin_y + self.scroll_y;
            let col = self.pixel_to_col(data_x);
            let row = self.pixel_to_row(data_y);
            let r = HitResult { hit_type: "cell".into(), row: Some(row), col: Some(col), group_id: None };
            return serde_json::to_string(&r).unwrap();
        }

        let r = HitResult { hit_type: "none".into(), row: None, col: None, group_id: None };
        serde_json::to_string(&r).unwrap()
    }

    // ─── Get cell screen rect (for editor positioning) ──────────

    pub fn cell_screen_rect(&mut self, row: u32, col: u32) -> String {
        self.rebuild_hidden();
        let m = self.metrics();
        let sx = self.col_pixel(col) - self.scroll_x + m.content_origin_x;
        let sy = self.row_pixel(row) - self.scroll_y + m.content_origin_y;
        let w = self.col_w(col);
        let h = self.row_h(row);
        format!(r#"{{"x":{},"y":{},"w":{},"h":{}}}"#, sx, sy, w, h)
    }

    // ─── Compute render frame ────────────────────────────────────

    pub fn render_frame(&mut self) -> String {
        self.rebuild_hidden();
        let m = self.metrics();
        let ox = m.content_origin_x;
        let oy = m.content_origin_y;
        let cw = self.viewport_w - ox;
        let ch = self.viewport_h - oy;

        // Visible columns
        let mut vis_cols: Vec<(u32, f64)> = Vec::new();
        {
            let mut px = 0.0_f64;
            let mut c = 0u32;
            while px + self.col_w(c) <= self.scroll_x {
                if !self.is_col_hidden(c) { px += self.col_w(c); }
                c += 1;
                if c > 999_999 { break; }
            }
            loop {
                if c > 999_999 { break; }
                if self.is_col_hidden(c) { c += 1; continue; }
                let sx = px - self.scroll_x + ox;
                if sx >= self.viewport_w { break; }
                vis_cols.push((c, sx));
                px += self.col_w(c);
                c += 1;
            }
        }

        // Visible rows
        let mut vis_rows: Vec<(u32, f64)> = Vec::new();
        {
            let mut py = 0.0_f64;
            let mut r = 0u32;
            while py + self.row_h(r) <= self.scroll_y {
                if !self.is_row_hidden(r) { py += self.row_h(r); }
                r += 1;
                if r > 999_999 { break; }
            }
            loop {
                if r > 999_999 { break; }
                if self.is_row_hidden(r) { r += 1; continue; }
                let sy = py - self.scroll_y + oy;
                if sy >= self.viewport_h { break; }
                vis_rows.push((r, sy));
                py += self.row_h(r);
                r += 1;
            }
        }

        // Cells
        let mut cells: Vec<VisibleCell> = Vec::new();
        for &(row, sy) in &vis_rows {
            let h = self.row_h(row);
            for &(col, sx) in &vis_cols {
                let w = self.col_w(col);
                let cd = self.cells.get(&(row, col));
                let selected = self.sel_row == row as i32 && self.sel_col == col as i32;
                let editing = self.edit_row == row as i32 && self.edit_col == col as i32;
                cells.push(VisibleCell {
                    sx,
                    sy,
                    w,
                    h,
                    text: cd.map(|c| c.text.clone()).unwrap_or_default(),
                    bg: if editing {
                        "#FFFFFF".into()
                    } else if selected {
                        "#DBEAFE".into()
                    } else {
                        cd.and_then(|c| c.bg_color.clone())
                            .unwrap_or("#FFFFFF".into())
                    },
                    fg: cd
                        .and_then(|c| c.fg_color.clone())
                        .unwrap_or("#1E293B".into()),
                    bold: cd.map(|c| c.bold).unwrap_or(false),
                    row,
                    col,
                    selected,
                    editing,
                });
            }
        }

        // Headers
        let col_headers: Vec<VisibleHeader> = vis_cols.iter().map(|&(c, sx)| {
            VisibleHeader {
                pos: sx, size: self.col_w(c),
                label: col_label(c), index: c,
                highlighted: self.sel_col == c as i32,
            }
        }).collect();

        let row_headers: Vec<VisibleHeader> = vis_rows.iter().map(|&(r, sy)| {
            VisibleHeader {
                pos: sy, size: self.row_h(r),
                label: (r + 1).to_string(), index: r,
                highlighted: self.sel_row == r as i32,
            }
        }).collect();

        // Group brackets
        let rg = self.row_groups.clone();
        let cg = self.col_groups.clone();

        let row_brackets: Vec<GroupBracket> = rg.iter().filter(|g| !self.ancestor_collapsed(g.id, &rg)).map(|g| {
            let (s, e) = self.group_pixel_range_row(g);
            GroupBracket {
                id: g.id, label: g.label.clone(),
                start: s - self.scroll_y + oy,
                end: e - self.scroll_y + oy,
                depth: g.depth, collapsed: g.collapsed, is_row: true,
            }
        }).filter(|b| b.end >= oy && b.start < self.viewport_h).collect();

        let col_brackets: Vec<GroupBracket> = cg.iter().filter(|g| !self.ancestor_collapsed(g.id, &cg)).map(|g| {
            let (s, e) = self.group_pixel_range_col(g);
            GroupBracket {
                id: g.id, label: g.label.clone(),
                start: s - self.scroll_x + ox,
                end: e - self.scroll_x + ox,
                depth: g.depth, collapsed: g.collapsed, is_row: false,
            }
        }).filter(|b| b.end >= ox && b.start < self.viewport_w).collect();

        let frame = RenderFrame { cells, col_headers, row_headers, row_brackets, col_brackets, metrics: m };
        serde_json::to_string(&frame).unwrap()
    }

    // ─── Stats ───────────────────────────────────────────────────

    pub fn cell_count(&self) -> u32 { self.cells.len() as u32 }
    pub fn group_count(&self) -> u32 { (self.row_groups.len() + self.col_groups.len()) as u32 }
}

fn col_label(mut c: u32) -> String {
    let mut s = String::new();
    loop {
        s.insert(0, (b'A' + (c % 26) as u8) as char);
        if c < 26 { break; }
        c = c / 26 - 1;
    }
    s
}
