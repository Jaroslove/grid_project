import { Component, output, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { GridWasmService } from '../../services/grid-wasm.service';

@Component({
  selector: 'app-grid-toolbar',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './grid-toolbar.component.html',
  styleUrl: './grid-toolbar.component.scss',
})
export class GridToolbarComponent {

}
