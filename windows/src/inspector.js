// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Inspector Panel — Phase 6.0

export class InspectorPanel {
  constructor(options) {
    this.container = options.container;
    this.onUpdateTransform = options.onUpdateTransform || (() => {});
    this.onUpdateEffect = options.onUpdateEffect || (() => {});
    this.onUpdateText = options.onUpdateText || (() => {});

    this.clip = null;
    this.trackIndex = -1;
    this.activeTab = 'adjust'; // 'adjust' | 'audio' | 'text'

    this.render();
  }

  setClip(clip, trackIndex) {
    this.clip = clip;
    this.trackIndex = trackIndex;
    if (clip && clip.mediaType === 'Text') {
      this.activeTab = 'text';
    } else if (clip && clip.mediaType === 'Audio') {
      this.activeTab = 'audio';
    } else {
      this.activeTab = 'adjust';
    }
    this.render();
  }

  getEffectValue(effectType, paramKey, defaultValue) {
    if (!this.clip || !this.clip.effects) return defaultValue;
    const eff = this.clip.effects.find(e => e.type === effectType);
    if (!eff || !eff.params || !eff.params[paramKey]) return defaultValue;
    return eff.params[paramKey].value !== undefined ? eff.params[paramKey].value : defaultValue;
  }

  render() {
    if (!this.container) return;
    this.container.innerHTML = '';

    if (!this.clip) {
      this.container.innerHTML = `
        <div class="inspector-empty">
          <div class="empty-icon">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="3" y="3" width="18" height="18" rx="2" stroke="currentColor"/>
              <line x1="3" y1="9" x2="21" y2="9" stroke="currentColor"/>
              <line x1="9" y1="21" x2="9" y2="9" stroke="currentColor"/>
            </svg>
          </div>
          <p class="empty-title">No Clip Selected</p>
          <p class="empty-desc">Select a clip on the timeline to inspect and edit its properties</p>
        </div>
      `;
      return;
    }

    // Clip Header
    const header = document.createElement('div');
    header.className = 'inspector-header';
    header.innerHTML = `
      <div class="clip-badge">${this.clip.mediaType || 'Video'}</div>
      <div class="clip-name" title="${this.clip.id}">${this.clip.textContent || this.clip.mediaRef || this.clip.id}</div>
      <div class="clip-meta tabular-nums">${this.clip.durationFrames} frames</div>
    `;
    this.container.appendChild(header);

    // Tab Navigation
    const tabs = document.createElement('div');
    tabs.className = 'inspector-tabs';

    const tabList = [
      { id: 'adjust', label: 'Adjust' },
      { id: 'audio', label: 'Audio' },
      { id: 'text', label: 'Text' },
    ];

    tabList.forEach(tab => {
      const btn = document.createElement('button');
      btn.className = `inspector-tab-btn ${this.activeTab === tab.id ? 'active' : ''}`;
      btn.textContent = tab.label;
      btn.addEventListener('click', () => {
        this.activeTab = tab.id;
        this.render();
      });
      tabs.appendChild(btn);
    });
    this.container.appendChild(tabs);

    // Tab Content Body
    const body = document.createElement('div');
    body.className = 'inspector-body';

    if (this.activeTab === 'adjust') {
      this.renderAdjustTab(body);
    } else if (this.activeTab === 'audio') {
      this.renderAudioTab(body);
    } else if (this.activeTab === 'text') {
      this.renderTextTab(body);
    }

    this.container.appendChild(body);
  }

  createSliderRow(label, min, max, step, value, onChange) {
    const row = document.createElement('div');
    row.className = 'inspector-row';

    const lbl = document.createElement('label');
    lbl.className = 'inspector-label';
    lbl.textContent = label;

    const input = document.createElement('input');
    input.type = 'range';
    input.className = 'inspector-slider';
    input.min = min;
    input.max = max;
    input.step = step;
    input.value = value;

    const valDisplay = document.createElement('span');
    valDisplay.className = 'inspector-val tabular-nums';
    valDisplay.textContent = Number(value).toFixed(2);

    input.addEventListener('input', (e) => {
      const val = parseFloat(e.target.value);
      valDisplay.textContent = val.toFixed(2);
      onChange(val);
    });

    row.appendChild(lbl);
    row.appendChild(input);
    row.appendChild(valDisplay);
    return row;
  }

  renderAdjustTab(parent) {
    // Transform Section
    const transformSec = document.createElement('div');
    transformSec.className = 'inspector-section';
    transformSec.innerHTML = '<div class="section-title">Transform & Opacity</div>';

    const tf = this.clip.transform || {
      centerX: 0.5,
      centerY: 0.5,
      width: 1.0,
      height: 1.0,
      rotation: 0.0,
    };
    const opacity = this.clip.opacity !== undefined ? this.clip.opacity : 1.0;

    const onTfChange = () => {
      this.onUpdateTransform({
        clipId: this.clip.id,
        centerX: tf.centerX,
        centerY: tf.centerY,
        width: tf.width,
        height: tf.height,
        rotation: tf.rotation,
        opacity: opacity,
      });
    };

    transformSec.appendChild(this.createSliderRow('Opacity', 0.0, 1.0, 0.05, opacity, (v) => {
      this.clip.opacity = v;
      onTfChange();
    }));

    transformSec.appendChild(this.createSliderRow('Scale X', 0.1, 3.0, 0.05, tf.width, (v) => {
      tf.width = v;
      onTfChange();
    }));

    transformSec.appendChild(this.createSliderRow('Scale Y', 0.1, 3.0, 0.05, tf.height, (v) => {
      tf.height = v;
      onTfChange();
    }));

    transformSec.appendChild(this.createSliderRow('Rotation', -180, 180, 1, tf.rotation, (v) => {
      tf.rotation = v;
      onTfChange();
    }));

    parent.appendChild(transformSec);

    // Tone Section
    const toneSec = document.createElement('div');
    toneSec.className = 'inspector-section';
    toneSec.innerHTML = '<div class="section-title">Tone Controls</div>';

    const exposure = this.getEffectValue('color.exposure', 'ev', 0.0);
    toneSec.appendChild(this.createSliderRow('Exposure', -4.0, 4.0, 0.1, exposure, (v) => {
      this.onUpdateEffect(this.clip.id, 'color.exposure', 'ev', v);
    }));

    const contrast = this.getEffectValue('color.contrast', 'amount', 0.0);
    toneSec.appendChild(this.createSliderRow('Contrast', -1.0, 1.0, 0.05, contrast, (v) => {
      this.onUpdateEffect(this.clip.id, 'color.contrast', 'amount', v);
    }));

    const highlights = this.getEffectValue('color.highlightsShadows', 'highlights', 0.0);
    toneSec.appendChild(this.createSliderRow('Highlights', -1.0, 1.0, 0.05, highlights, (v) => {
      this.onUpdateEffect(this.clip.id, 'color.highlightsShadows', 'highlights', v);
    }));

    const shadows = this.getEffectValue('color.highlightsShadows', 'shadows', 0.0);
    toneSec.appendChild(this.createSliderRow('Shadows', -1.0, 1.0, 0.05, shadows, (v) => {
      this.onUpdateEffect(this.clip.id, 'color.highlightsShadows', 'shadows', v);
    }));

    parent.appendChild(toneSec);

    // Stylize Section
    const stylizeSec = document.createElement('div');
    stylizeSec.className = 'inspector-section';
    stylizeSec.innerHTML = '<div class="section-title">Stylize & Blur</div>';

    const vignette = this.getEffectValue('stylize.vignette', 'amount', 0.0);
    stylizeSec.appendChild(this.createSliderRow('Vignette', 0.0, 1.0, 0.05, vignette, (v) => {
      this.onUpdateEffect(this.clip.id, 'stylize.vignette', 'amount', v);
    }));

    const blur = this.getEffectValue('blur.gaussian', 'radius', 0.0);
    stylizeSec.appendChild(this.createSliderRow('Blur Radius', 0.0, 40.0, 1.0, blur, (v) => {
      this.onUpdateEffect(this.clip.id, 'blur.gaussian', 'radius', v);
    }));

    parent.appendChild(stylizeSec);
  }

  renderAudioTab(parent) {
    const audioSec = document.createElement('div');
    audioSec.className = 'inspector-section';
    audioSec.innerHTML = '<div class="section-title">Audio Levels</div>';

    const volume = this.clip.volume !== undefined ? this.clip.volume : 1.0;
    audioSec.appendChild(this.createSliderRow('Volume', 0.0, 2.0, 0.05, volume, (v) => {
      this.clip.volume = v;
      this.onUpdateEffect(this.clip.id, 'audio.volume', 'level', v);
    }));

    parent.appendChild(audioSec);
  }

  renderTextTab(parent) {
    const textSec = document.createElement('div');
    textSec.className = 'inspector-section';
    textSec.innerHTML = '<div class="section-title">Text Content & Style</div>';

    const row = document.createElement('div');
    row.style.marginBottom = '12px';

    const label = document.createElement('label');
    label.className = 'inspector-label';
    label.style.display = 'block';
    label.style.marginBottom = '6px';
    label.textContent = 'Title Content';

    const textarea = document.createElement('textarea');
    textarea.className = 'inspector-textarea';
    textarea.value = this.clip.textContent || '';
    textarea.rows = 3;

    row.appendChild(label);
    row.appendChild(textarea);
    textSec.appendChild(row);

    const fontSize = (this.clip.textStyle && this.clip.textStyle.fontSize) ? this.clip.textStyle.fontSize : 72.0;

    let currentSize = fontSize;
    textarea.addEventListener('input', (e) => {
      this.clip.textContent = e.target.value;
      this.onUpdateText(this.clip.id, e.target.value, currentSize);
    });

    textSec.appendChild(this.createSliderRow('Font Size', 16, 140, 2, fontSize, (v) => {
      currentSize = v;
      this.onUpdateText(this.clip.id, textarea.value, v);
    }));

    parent.appendChild(textSec);
  }
}
