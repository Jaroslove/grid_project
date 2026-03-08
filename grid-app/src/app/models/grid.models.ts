export interface VC {
  sx: number; sy: number; w: number; h: number;
  text: string; bg: string; fg: string; bold: boolean;
  row: number; col: number; sel: boolean; edit: boolean;
  ct: number; ta: number; ind: number;
}
export interface VH { pos: number; sz: number; lbl: string; idx: number; hi: boolean; }
export interface VB { id: number; lbl: string; s: number; e: number; d: number; collapsed: boolean; }
export interface SB { vis: boolean; tx: number; ty: number; tw: number; th: number; bx: number; by: number; bw: number; bh: number; }
export interface Met {
  ox: number; oy: number; fw: number; fh: number;
  fc: number; fr: number; rhw: number; chh: number;
  rbd: number; cbd: number; bs: number; sbs: number;
}
export interface Frame {
  cells: VC[]; ch: VH[]; rh: VH[];
  rb: VB[]; cb: VB[];
  m: Met; hs: SB; vs: SB;
}
export interface HitResult { type: string; row?: number; col?: number; key?: string; }
export interface CellRect { x: number; y: number; w: number; h: number; }
export interface PivotField { name: string; }
export interface ValueField { name: string; agg: 'Sum'|'Count'|'Average'|'Min'|'Max'; label?: string; }
export interface PivotConfig {
  row_fields: PivotField[]; col_fields: PivotField[];
  value_fields: ValueField[];
  show_row_subtotals: boolean; show_col_subtotals: boolean; show_grand_totals: boolean;
}
