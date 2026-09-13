// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Timeline Canvas Engine — Phase 6.0

export class TimelineEngine {
  constructor(options) {
    this.container = options.container;
    this.rulerCanvas = options.rulerCanvas;
    this.tracksCanvas = options.tracksCanvas;
    this.headersContainer = options.headersContainer;
    this.onSeek = options.onSeek || (() => {});
    this.onSelectClip = options.onSelectClip || (() => {});
    this.onTimelineMutate = options.onTimelineMutate || (() => {});

    this.timeline = null;
    this.currentFrame = 0;
    this.fps = 30;
    this.pxPerFrame = 2.0; // zoom level
    this.minPxPerFrame = 0.2;
    this.maxPxPerFrame = 12.0;

    this.selectedClipId = null;      // primary selection (drives the inspector)
    this.selectedClipIds = new Set(); // full multi-selection
    this.hoveredClipId = null;
    this.trackHeight = 44;
    this.rulerHeight = 26;

    // Interaction state
    this.isDraggingPlayhead = false;
    this.activeDrag = null; // { type: 'move' | 'trim-left' | 'trim-right', clipId, trackIndex, startX, startY, origStartFrame, origDuration }
    this.lasso = null;      // { x0, y0, x1, y1 } rubberband selection rectangle
    this.snapGuideFrame = null; // frame where a magnetic snap guide is drawn during a move
    this.snapThresholdPx = 8;

    this.initEvents();
  }

  setTimeline(timeline, currentFrame) {
    this.timeline = timeline;
    if (timeline && timeline.fps) {
      this.fps = timeline.fps;
    }
    if (currentFrame !== undefined) {
      this.currentFrame = currentFrame;
    }
    this.render();
  }

  setFrame(frame) {
    this.currentFrame = frame;
    this.renderRuler();
    this.renderTracks();
  }

  setZoom(zoomFactor) {
    this.pxPerFrame = Math.max(this.minPxPerFrame, Math.min(this.maxPxPerFrame, zoomFactor));
    this.render();
  }

  getSelectedClip() {
    if (!this.timeline || !this.selectedClipId) return null;
    for (const track of this.timeline.tracks) {
      for (const clip of track.clips) {
        if (clip.id === this.selectedClipId) {
          return { clip, track };
        }
      }
    }
    return null;
  }

  formatTimecode(frame, fps) {
    const f = Math.floor(frame % fps);
    const totalSec = Math.floor(frame / fps);
    const s = totalSec % 60;
    const m = Math.floor(totalSec / 60) % 60;
    const h = Math.floor(totalSec / 3600);
    return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}:${String(f).padStart(2, '0')}`;
  }

  render() {
    this.renderHeaders();
    this.renderRuler();
    this.renderTracks();
  }

  renderHeaders() {
    if (!this.headersContainer || !this.timeline) return;
    this.headersContainer.innerHTML = '';

    this.timeline.tracks.forEach((track, idx) => {
      const header = document.createElement('div');
      header.className = 'track-header';
      header.style.height = `${this.trackHeight}px`;

      const typeBadge = document.createElement('span');
      typeBadge.className = `track-badge track-badge-${track.type.toLowerCase()}`;
      typeBadge.textContent = track.type.substring(0, 1).toUpperCase();

      const title = document.createElement('span');
      title.className = 'track-title';
      title.textContent = track.name || `Track ${idx + 1} (${track.type})`;

      const controls = document.createElement('div');
      controls.className = 'track-controls';

      const muteBtn = document.createElement('button');
      muteBtn.className = `track-btn ${track.muted ? 'active' : ''}`;
      muteBtn.textContent = 'M';
      muteBtn.title = 'Mute Track';

      const lockBtn = document.createElement('button');
      lockBtn.className = `track-btn ${track.syncLocked ? 'active' : ''}`;
      lockBtn.textContent = 'L';
      lockBtn.title = 'Sync Lock';

      controls.appendChild(muteBtn);
      controls.appendChild(lockBtn);

      header.appendChild(typeBadge);
      header.appendChild(title);
      header.appendChild(controls);

      this.headersContainer.appendChild(header);
    });
  }

  renderRuler() {
    if (!this.rulerCanvas) return;
    const ctx = this.rulerCanvas.getContext('2d');
    const width = this.rulerCanvas.width;
    const height = this.rulerCanvas.height;

    ctx.clearRect(0, 0, width, height);

    // Background
    ctx.fillStyle = '#16171a';
    ctx.fillRect(0, 0, width, height);

    // Bottom divider
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.12)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, height - 0.5);
    ctx.lineTo(width, height - 0.5);
    ctx.stroke();

    // Time ticks
    const stepSeconds = this.pxPerFrame * this.fps < 60 ? 5 : (this.pxPerFrame * this.fps < 150 ? 2 : 1);
    const stepFrames = stepSeconds * this.fps;

    ctx.font = '10px -apple-system, BlinkMacSystemFont, "SF Mono", monospace';
    ctx.fillStyle = '#71717a';

    const maxFrames = Math.ceil(width / this.pxPerFrame);
    for (let f = 0; f <= maxFrames; f += stepFrames) {
      const x = f * this.pxPerFrame;

      // Major tick
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.25)';
      ctx.beginPath();
      ctx.moveTo(x, height - 12);
      ctx.lineTo(x, height);
      ctx.stroke();

      // Label
      const tc = this.formatTimecode(f, this.fps);
      ctx.fillText(tc, x + 4, height - 6);

      // Sub-ticks
      const subSteps = 4;
      for (let s = 1; s < subSteps; s++) {
        const subX = x + (s * (stepFrames / subSteps)) * this.pxPerFrame;
        if (subX > width) break;
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
        ctx.beginPath();
        ctx.moveTo(subX, height - 6);
        ctx.lineTo(subX, height);
        ctx.stroke();
      }
    }

    // Playhead indicator on ruler
    const playheadX = this.currentFrame * this.pxPerFrame;
    ctx.fillStyle = '#ff524d';
    ctx.beginPath();
    ctx.moveTo(playheadX - 6, 0);
    ctx.lineTo(playheadX + 6, 0);
    ctx.lineTo(playheadX + 6, 14);
    ctx.lineTo(playheadX, 22);
    ctx.lineTo(playheadX - 6, 14);
    ctx.closePath();
    ctx.fill();
  }

  renderTracks() {
    if (!this.tracksCanvas || !this.timeline) return;
    const ctx = this.tracksCanvas.getContext('2d');
    const width = this.tracksCanvas.width;
    const height = this.tracksCanvas.height;

    ctx.clearRect(0, 0, width, height);

    // Track lane backgrounds
    this.timeline.tracks.forEach((track, trackIdx) => {
      const y = trackIdx * this.trackHeight;
      ctx.fillStyle = trackIdx % 2 === 0 ? '#111215' : '#141519';
      ctx.fillRect(0, y, width, this.trackHeight);

      // Track bottom border
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.06)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(0, y + this.trackHeight - 0.5);
      ctx.lineTo(width, y + this.trackHeight - 0.5);
      ctx.stroke();

      // Render clips on track
      track.clips.forEach(clip => {
        // While moving, preview the dragged (and co-selected) clips at their target position.
        let displayStartFrame = clip.startFrame;
        if (this.activeDrag && this.activeDrag.type === 'move' && this.activeDrag.moves) {
          const m = this.activeDrag.moves.find(mv => mv.clipId === clip.id);
          if (m && this.activeDrag.delta !== undefined) {
            displayStartFrame = Math.max(0, m.origStartFrame + this.activeDrag.delta);
          }
        }
        const clipX = displayStartFrame * this.pxPerFrame;
        const clipW = clip.durationFrames * this.pxPerFrame;
        const clipY = y + 4;
        const clipH = this.trackHeight - 8;

        const isSelected = this.isClipSelected(clip.id);
        const isHovered = clip.id === this.hoveredClipId;

        // Clip body color
        let color = '#1d5878'; // video default
        if (clip.mediaType === 'Audio' || track.type === 'Audio') color = '#2e7765';
        else if (clip.mediaType === 'Text' || track.type === 'Text') color = '#546e7a';
        else if (clip.mediaType === 'Image' || track.type === 'Image') color = '#715486';

        ctx.save();
        ctx.fillStyle = color;
        ctx.strokeStyle = isSelected ? '#ffffff' : (isHovered ? 'rgba(255, 255, 255, 0.4)' : 'rgba(0, 0, 0, 0.3)');
        ctx.lineWidth = isSelected ? 2 : 1;

        // Draw rounded rect
        const r = 4;
        ctx.beginPath();
        ctx.roundRect(clipX, clipY, Math.max(clipW, 2), clipH, r);
        ctx.fill();
        ctx.stroke();

        // Clip label
        if (clipW > 30) {
          ctx.font = '11px -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif';
          ctx.fillStyle = '#f0f0f2';
          const label = clip.textContent || clip.mediaRef || clip.id;
          ctx.save();
          ctx.beginPath();
          ctx.rect(clipX + 6, clipY, clipW - 12, clipH);
          ctx.clip();
          ctx.fillText(label, clipX + 8, clipY + 18);

          // Subtitle / duration
          ctx.font = '9px "SF Mono", monospace';
          ctx.fillStyle = 'rgba(255, 255, 255, 0.6)';
          ctx.fillText(`${clip.durationFrames}f`, clipX + 8, clipY + 30);
          ctx.restore();
        }

        // Trim edge handles if selected
        if (isSelected && clipW > 16) {
          ctx.fillStyle = '#ffffff';
          // Left handle
          ctx.fillRect(clipX, clipY + 8, 3, clipH - 16);
          // Right handle
          ctx.fillRect(clipX + clipW - 3, clipY + 8, 3, clipH - 16);
        }

        ctx.restore();
      });
    });

    // Magnetic snap guide during a move drag
    if (this.snapGuideFrame !== null) {
      const snapX = this.snapGuideFrame * this.pxPerFrame;
      ctx.strokeStyle = '#ffd54a';
      ctx.lineWidth = 1;
      ctx.setLineDash([4, 3]);
      ctx.beginPath();
      ctx.moveTo(snapX, 0);
      ctx.lineTo(snapX, height);
      ctx.stroke();
      ctx.setLineDash([]);
    }

    // Playhead line through tracks
    const playheadX = this.currentFrame * this.pxPerFrame;
    ctx.strokeStyle = '#ff524d';
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(playheadX, 0);
    ctx.lineTo(playheadX, height);
    ctx.stroke();

    // Rubberband lasso rectangle
    if (this.lasso) {
      const lx = Math.min(this.lasso.x0, this.lasso.x1);
      const ly = Math.min(this.lasso.y0, this.lasso.y1);
      const lw = Math.abs(this.lasso.x1 - this.lasso.x0);
      const lh = Math.abs(this.lasso.y1 - this.lasso.y0);
      ctx.fillStyle = 'rgba(74, 158, 255, 0.15)';
      ctx.strokeStyle = 'rgba(74, 158, 255, 0.8)';
      ctx.lineWidth = 1;
      ctx.fillRect(lx, ly, lw, lh);
      ctx.strokeRect(lx + 0.5, ly + 0.5, lw, lh);
    }
  }

  isClipSelected(clipId) {
    return clipId === this.selectedClipId || this.selectedClipIds.has(clipId);
  }

  // Clips whose track row and frame span intersect a pixel-space rectangle.
  getClipsInRect(x0, y0, x1, y1) {
    const result = [];
    if (!this.timeline) return result;
    const left = Math.min(x0, x1);
    const right = Math.max(x0, x1);
    const top = Math.min(y0, y1);
    const bottom = Math.max(y0, y1);
    this.timeline.tracks.forEach((track, trackIdx) => {
      const trackTop = trackIdx * this.trackHeight;
      const trackBottom = trackTop + this.trackHeight;
      if (trackBottom < top || trackTop > bottom) return;
      for (const clip of track.clips) {
        const cx = clip.startFrame * this.pxPerFrame;
        const cw = clip.durationFrames * this.pxPerFrame;
        if (cx + cw >= left && cx <= right) {
          result.push({ clip, trackIndex: trackIdx });
        }
      }
    });
    return result;
  }

  // Snap a candidate start frame to nearby clip edges, the playhead, or frame 0.
  // Returns { startFrame, guideFrame } — guideFrame is null when nothing snapped.
  computeSnap(candidateStart, durationFrames, excludeIds) {
    const thresholdFrames = this.snapThresholdPx / this.pxPerFrame;
    const candidateEnd = candidateStart + durationFrames;

    const points = [0, this.currentFrame];
    if (this.timeline) {
      for (const track of this.timeline.tracks) {
        for (const clip of track.clips) {
          if (excludeIds.has(clip.id)) continue;
          points.push(clip.startFrame);
          points.push(clip.startFrame + clip.durationFrames);
        }
      }
    }

    let best = null; // { adjustedStart, guideFrame, dist }
    for (const p of points) {
      const dStart = Math.abs(candidateStart - p);
      if (dStart <= thresholdFrames && (!best || dStart < best.dist)) {
        best = { adjustedStart: p, guideFrame: p, dist: dStart };
      }
      const dEnd = Math.abs(candidateEnd - p);
      if (dEnd <= thresholdFrames && (!best || dEnd < best.dist)) {
        best = { adjustedStart: p - durationFrames, guideFrame: p, dist: dEnd };
      }
    }

    if (best && best.adjustedStart >= 0) {
      return { startFrame: best.adjustedStart, guideFrame: best.guideFrame };
    }
    return { startFrame: candidateStart, guideFrame: null };
  }

  getHitClip(mouseX, mouseY) {
    if (!this.timeline) return null;
    const trackIdx = Math.floor(mouseY / this.trackHeight);
    if (trackIdx < 0 || trackIdx >= this.timeline.tracks.length) return null;

    const track = this.timeline.tracks[trackIdx];
    for (const clip of track.clips) {
      const x = clip.startFrame * this.pxPerFrame;
      const w = clip.durationFrames * this.pxPerFrame;
      if (mouseX >= x && mouseX <= x + w) {
        // Check trim edges
        const isLeftEdge = Math.abs(mouseX - x) <= 8;
        const isRightEdge = Math.abs(mouseX - (x + w)) <= 8;
        return {
          clip,
          trackIndex: trackIdx,
          isLeftEdge,
          isRightEdge,
        };
      }
    }
    return null;
  }

  initEvents() {
    // Ruler mouse drag for playhead seek
    this.rulerCanvas.addEventListener('mousedown', (e) => {
      const rect = this.rulerCanvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const frame = Math.max(0, Math.round(x / this.pxPerFrame));
      this.isDraggingPlayhead = true;
      this.currentFrame = frame;
      this.onSeek(frame, 'interactiveScrub');
      this.renderRuler();
      this.renderTracks();
    });

    window.addEventListener('mousemove', (e) => {
      if (this.isDraggingPlayhead) {
        const rect = this.rulerCanvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const frame = Math.max(0, Math.round(x / this.pxPerFrame));
        this.currentFrame = frame;
        this.onSeek(frame, 'interactiveScrub');
        this.renderRuler();
        this.renderTracks();
      } else if (this.lasso) {
        // Update rubberband rectangle
        const rect = this.tracksCanvas.getBoundingClientRect();
        this.lasso.x1 = e.clientX - rect.left;
        this.lasso.y1 = e.clientY - rect.top;
        const hits = this.getClipsInRect(this.lasso.x0, this.lasso.y0, this.lasso.x1, this.lasso.y1);
        this.selectedClipIds = new Set(hits.map(h => h.clip.id));
        this.renderTracks();
      } else if (this.activeDrag) {
        // Dragging / trimming clip
        const rect = this.tracksCanvas.getBoundingClientRect();
        const deltaX = (e.clientX - rect.left) - this.activeDrag.startX;
        const deltaFrames = Math.round(deltaX / this.pxPerFrame);

        if (this.activeDrag.type === 'move') {
          // Preview move for the primary clip, with magnetic snapping.
          const rawStart = Math.max(0, this.activeDrag.origStartFrame + deltaFrames);
          const excludeIds = new Set(this.activeDrag.moves.map(m => m.clipId));
          const snap = this.computeSnap(rawStart, this.activeDrag.origDuration, excludeIds);
          this.activeDrag.targetStartFrame = snap.startFrame;
          this.activeDrag.delta = snap.startFrame - this.activeDrag.origStartFrame;
          this.snapGuideFrame = snap.guideFrame;
          this.renderTracks();
        } else if (this.activeDrag.type === 'trim-left') {
          const maxDelta = this.activeDrag.origDuration - 1;
          const clampedDelta = Math.min(maxDelta, Math.max(-this.activeDrag.origStartFrame, deltaFrames));
          this.activeDrag.delta = clampedDelta;
        } else if (this.activeDrag.type === 'trim-right') {
          const minDelta = -(this.activeDrag.origDuration - 1);
          this.activeDrag.delta = Math.max(minDelta, deltaFrames);
        }
      } else {
        // Hover inspection
        const rect = this.tracksCanvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        if (x >= 0 && x <= rect.width && y >= 0 && y <= rect.height) {
          const hit = this.getHitClip(x, y);
          if (hit) {
            this.hoveredClipId = hit.clip.id;
            if (hit.isLeftEdge || hit.isRightEdge) {
              this.tracksCanvas.style.cursor = 'ew-resize';
            } else {
              this.tracksCanvas.style.cursor = 'pointer';
            }
          } else {
            this.hoveredClipId = null;
            this.tracksCanvas.style.cursor = 'default';
          }
          this.renderTracks();
        }
      }
    });

    window.addEventListener('mouseup', async () => {
      if (this.isDraggingPlayhead) {
        this.isDraggingPlayhead = false;
      }
      if (this.lasso) {
        const hits = this.getClipsInRect(this.lasso.x0, this.lasso.y0, this.lasso.x1, this.lasso.y1);
        this.selectedClipIds = new Set(hits.map(h => h.clip.id));
        // Primary selection = first hit (drives the inspector); null if none.
        if (hits.length > 0) {
          this.selectedClipId = hits[0].clip.id;
          this.onSelectClip(hits[0].clip, hits[0].trackIndex);
        } else {
          this.selectedClipId = null;
          this.onSelectClip(null, -1);
        }
        this.lasso = null;
        this.renderTracks();
        return;
      }
      if (this.activeDrag) {
        const drag = this.activeDrag;
        this.activeDrag = null;
        this.snapGuideFrame = null;

        if (drag.type === 'move' && drag.delta) {
          // Move every selected clip by the same snapped delta.
          const moves = drag.moves
            .filter(m => m.origStartFrame + drag.delta >= 0)
            .map(m => ({ clipId: m.clipId, trackIndex: m.trackIndex, delta: drag.delta }));
          if (moves.length > 0) {
            this.onTimelineMutate('move', { moves });
          }
          this.renderTracks();
        } else if (drag.type === 'trim-left' && drag.delta) {
          this.onTimelineMutate('trim', {
            clipId: drag.clipId,
            edge: 'left',
            delta: drag.delta,
          });
        } else if (drag.type === 'trim-right' && drag.delta) {
          this.onTimelineMutate('trim', {
            clipId: drag.clipId,
            edge: 'right',
            delta: drag.delta,
          });
        }
      }
    });

    // Tracks canvas clip selection & drag start
    this.tracksCanvas.addEventListener('mousedown', (e) => {
      const rect = this.tracksCanvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      const additive = e.ctrlKey || e.metaKey;

      const hit = this.getHitClip(x, y);
      if (hit) {
        if (additive) {
          // Ctrl/Cmd+click toggles the clip in the multi-selection (no drag).
          if (this.selectedClipIds.has(hit.clip.id)) {
            this.selectedClipIds.delete(hit.clip.id);
            if (this.selectedClipId === hit.clip.id) {
              this.selectedClipId = null;
              this.onSelectClip(null, -1);
            }
          } else {
            this.selectedClipIds.add(hit.clip.id);
            this.selectedClipId = hit.clip.id;
            this.onSelectClip(hit.clip, hit.trackIndex);
          }
          this.renderTracks();
          return;
        }

        // Plain click: keep the selection if this clip is already part of it
        // (so a drag moves the whole group); otherwise select just this clip.
        if (!this.selectedClipIds.has(hit.clip.id)) {
          this.selectedClipIds = new Set([hit.clip.id]);
        }
        this.selectedClipId = hit.clip.id;
        this.onSelectClip(hit.clip, hit.trackIndex);

        if (hit.isLeftEdge) {
          this.activeDrag = {
            type: 'trim-left',
            clipId: hit.clip.id,
            startX: x,
            origDuration: hit.clip.durationFrames,
            origStartFrame: hit.clip.startFrame,
            delta: 0,
          };
        } else if (hit.isRightEdge) {
          this.activeDrag = {
            type: 'trim-right',
            clipId: hit.clip.id,
            startX: x,
            origDuration: hit.clip.durationFrames,
            origStartFrame: hit.clip.startFrame,
            delta: 0,
          };
        } else {
          // Collect every selected clip so the move applies to the whole group.
          const moves = [];
          this.timeline.tracks.forEach((track, trackIdx) => {
            for (const clip of track.clips) {
              if (this.selectedClipIds.has(clip.id)) {
                moves.push({ clipId: clip.id, trackIndex: trackIdx, origStartFrame: clip.startFrame });
              }
            }
          });
          this.activeDrag = {
            type: 'move',
            clipId: hit.clip.id,
            trackIndex: hit.trackIndex,
            startX: x,
            origStartFrame: hit.clip.startFrame,
            origDuration: hit.clip.durationFrames,
            moves,
            delta: 0,
          };
        }
      } else {
        // Empty space: clear selection and begin a rubberband lasso.
        this.selectedClipId = null;
        this.selectedClipIds = new Set();
        this.onSelectClip(null, -1);
        this.lasso = { x0: x, y0: y, x1: x, y1: y };
      }
      this.renderTracks();
    });

    // Zoom on Ctrl + Wheel
    this.container.addEventListener('wheel', (e) => {
      if (e.ctrlKey) {
        e.preventDefault();
        const zoomDelta = e.deltaY < 0 ? 1.15 : 0.85;
        this.setZoom(this.pxPerFrame * zoomDelta);
      }
    }, { passive: false });
  }
}
