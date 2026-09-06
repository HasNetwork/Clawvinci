// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Media Panel Asset Browser — Phase 6.0

export class MediaPanel {
  constructor(options) {
    this.container = options.container;
    this.onImport = options.onImport || (() => {});
    this.onAddToTimeline = options.onAddToTimeline || (() => {});

    this.mediaItems = [];
    this.searchQuery = '';

    this.render();
  }

  setItems(items) {
    this.mediaItems = items || [];
    this.render();
  }

  render() {
    if (!this.container) return;
    this.container.innerHTML = '';

    // Header with Title & Import Button
    const header = document.createElement('div');
    header.className = 'media-panel-header';

    const titleArea = document.createElement('div');
    titleArea.className = 'media-panel-title';
    titleArea.innerHTML = `<span>Project Media</span><span class="media-count">${this.mediaItems.length}</span>`;

    const importBtn = document.createElement('button');
    importBtn.className = 'media-import-btn';
    importBtn.innerHTML = `
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 5v14M5 12h14"/>
      </svg>
      <span>Import</span>
    `;
    importBtn.addEventListener('click', () => {
      this.promptImport();
    });

    header.appendChild(titleArea);
    header.appendChild(importBtn);
    this.container.appendChild(header);

    // Search bar
    const searchWrap = document.createElement('div');
    searchWrap.className = 'media-search-wrap';
    searchWrap.innerHTML = `
      <svg class="search-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
    `;
    const searchInput = document.createElement('input');
    searchInput.type = 'text';
    searchInput.className = 'media-search-input';
    searchInput.placeholder = 'Search media...';
    searchInput.value = this.searchQuery;
    searchInput.addEventListener('input', (e) => {
      this.searchQuery = e.target.value.toLowerCase();
      this.renderGrid(gridContainer);
    });
    searchWrap.appendChild(searchInput);
    this.container.appendChild(searchWrap);

    // Grid of media assets
    const gridContainer = document.createElement('div');
    gridContainer.className = 'media-grid';
    this.renderGrid(gridContainer);
    this.container.appendChild(gridContainer);
  }

  renderGrid(grid) {
    grid.innerHTML = '';

    const filtered = this.mediaItems.filter(item => {
      if (!this.searchQuery) return true;
      return item.name.toLowerCase().includes(this.searchQuery);
    });

    if (filtered.length === 0) {
      grid.innerHTML = `
        <div class="media-empty">
          <p class="empty-title">No Media Found</p>
          <p class="empty-desc">Click "Import" to add video, audio, or images to your project</p>
        </div>
      `;
      return;
    }

    filtered.forEach(item => {
      const card = document.createElement('div');
      card.className = 'media-card';

      // Thumbnail placeholder with icon
      const thumb = document.createElement('div');
      thumb.className = `media-thumb media-thumb-${item.mediaType || 'video'}`;

      let iconSvg = '';
      if (item.mediaType === 'audio') {
        iconSvg = `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>`;
      } else {
        iconSvg = `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="20" height="20" rx="2.18"/><line x1="7" y1="2" x2="7" y2="22"/><line x1="17" y1="2" x2="17" y2="22"/><line x1="2" y1="12" x2="22" y2="12"/></svg>`;
      }

      thumb.innerHTML = `
        <div class="thumb-icon">${iconSvg}</div>
        <div class="thumb-duration tabular-nums">${item.durationSeconds.toFixed(1)}s</div>
      `;

      const info = document.createElement('div');
      info.className = 'media-info';

      const name = document.createElement('div');
      name.className = 'media-name';
      name.textContent = item.name;
      name.title = item.name;

      const meta = document.createElement('div');
      meta.className = 'media-meta tabular-nums';
      if (item.width && item.height) {
        meta.textContent = `${item.width}x${item.height} • ${item.fps ? item.fps + 'fps' : '30fps'}`;
      } else {
        meta.textContent = `${item.mediaType.toUpperCase()}`;
      }

      info.appendChild(name);
      info.appendChild(meta);

      card.appendChild(thumb);
      card.appendChild(info);

      card.addEventListener('dblclick', () => {
        this.onAddToTimeline(item);
      });

      grid.appendChild(card);
    });
  }

  promptImport() {
    const input = prompt('Enter media file path to import (e.g. C:/Videos/scene.mp4):', 'sample-media-3.mp4');
    if (input) {
      this.onImport(input.trim());
    }
  }
}
