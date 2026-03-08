export interface VCell {
  sx: number; sy: number; w: number; h: number;
  text: string; bg: string; fg: string; bold: boolean;
  row: number; col: number; selected: boolean; editing: boolean;
  cell_type: number; text_align: number; indent: number;
}
export interface VHeader { pos: number; size: number; label: string; index: number; highlighted: boolean; }
export interface VBracket { id: number; label: string; start: number; end: number; depth: number; collapsed: boolean; }
export interface SBar { visible: boolean; tx: number; ty: number; tw: number; th: number; bx: number; by: number; bw: number; bh: number; }
export interface Metrics { ox: number; oy: number; fw: number; fh: number; fc: number; fr: number; rhw: number; chh: number; bd: number; bs: number; sbs: number; }
export interface RenderFrame { cells: VCell[]; ch: VHeader[]; rh: VHeader[]; rb: VBracket[]; m: Metrics; hs: SBar; vs: SBar; }
export interface HitResult { type: string; row?: number; col?: number; key?: string; }
export interface CellRect { x: number; y: number; w: number; h: number; }
export interface PivotField { name: string; }
export interface ValueField { name: string; agg: 'Sum'|'Count'|'Average'|'Min'|'Max'; label?: string; }
export interface PivotConfig { row_fields: PivotField[]; col_fields: PivotField[]; value_fields: ValueField[]; show_row_subtotals: boolean; show_col_subtotals: boolean; show_grand_totals: boolean; }
