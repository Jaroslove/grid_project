import { Component, ViewChild } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { GridToolbarComponent } from './components/grid-toolbar/grid-toolbar.component';
import { GridCanvasComponent } from './components/grid-canvas/grid-canvas.component';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, FormsModule, GridToolbarComponent, GridCanvasComponent],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss',
})
export class AppComponent {
  @ViewChild(GridCanvasComponent) gridCanvas!: GridCanvasComponent;

  onToolbarRender(): void {
    this.gridCanvas?.triggerRender();
  }
}
