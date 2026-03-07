export interface VisibleCell {
    sx: number;
    sy: number;
    w: number;
    h: number;
    text: string;
    bg: string;
    fg: string;
    bold: boolean;
    row: number;
    col: number;
    selected: boolean;
    editing: boolean;
  }
  
  export interface VisibleHeader {
    pos: number;
    size: number;
    label: string;
    index: number;
    highlighted: boolean;
  }
  
  export interface GroupBracket {
    id: number;
    label: string;
    start: number;
    end: number;
    depth: number;
    collapsed: boolean;
    is_row: boolean;
  }
  
  export interface GridMetrics {
    row_header_width: number;
    col_header_height: number;
    group_cols_depth: number;
    group_rows_depth: number;
    bracket_size: number;
    content_origin_x: number;
    content_origin_y: number;
  }
  
  export interface RenderFrame {
    cells: VisibleCell[];
    col_headers: VisibleHeader[];
    row_headers: VisibleHeader[];
    row_brackets: GroupBracket[];
    col_brackets: GroupBracket[];
    metrics: GridMetrics;
  }
  
  export interface HitResult {
    type: string;
    row?: number;
    col?: number;
    group_id?: number;
  }
  
  export interface CellRect {
    x: number;
    y: number;
    w: number;
    h: number;
  }
  