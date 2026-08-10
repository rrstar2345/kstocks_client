<script lang="ts">
  // Minimal dependency-free candlestick chart, drawn on a <canvas>. No
  // charting library required — keeps the widget grid light. Redraws
  // whenever `bars` changes or the container resizes.
  import type { OhlcBar } from "$lib/types";

  let { bars, height = 220 }: { bars: OhlcBar[]; height?: number } = $props();

  let canvasEl: HTMLCanvasElement;
  let containerEl: HTMLDivElement;

  function readColor(varName: string): string {
    return getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
  }

  function draw() {
    if (!canvasEl || !containerEl) return;
    const dpr = window.devicePixelRatio || 1;
    const width = containerEl.clientWidth;

    canvasEl.width = width * dpr;
    canvasEl.height = height * dpr;
    canvasEl.style.width = `${width}px`;
    canvasEl.style.height = `${height}px`;

    const ctx = canvasEl.getContext("2d");
    if (!ctx) return;
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, width, height);

    if (bars.length === 0) {
      ctx.fillStyle = readColor("--color-text-muted");
      ctx.font = "13px var(--font-sans)";
      ctx.fillText("No data yet", 12, height / 2);
      return;
    }

    const padding = { top: 10, bottom: 10, left: 4, right: 4 };
    const plotW = width - padding.left - padding.right;
    const plotH = height - padding.top - padding.bottom;

    const highs = bars.map((b) => b.high);
    const lows = bars.map((b) => b.low);
    const max = Math.max(...highs);
    const min = Math.min(...lows);
    const range = max - min || 1;

    const yFor = (price: number) => padding.top + plotH * (1 - (price - min) / range);
    const slot = plotW / bars.length;
    const candleW = Math.max(1, Math.min(10, slot * 0.6));

    const positiveColor = readColor("--color-positive");
    const negativeColor = readColor("--color-negative");

    bars.forEach((bar, i) => {
      const x = padding.left + i * slot + slot / 2;
      const isUp = bar.close >= bar.open;
      ctx.strokeStyle = isUp ? positiveColor : negativeColor;
      ctx.fillStyle = isUp ? positiveColor : negativeColor;

      // wick
      ctx.beginPath();
      ctx.moveTo(x, yFor(bar.high));
      ctx.lineTo(x, yFor(bar.low));
      ctx.lineWidth = 1;
      ctx.stroke();

      // body
      const bodyTop = yFor(Math.max(bar.open, bar.close));
      const bodyBottom = yFor(Math.min(bar.open, bar.close));
      const bodyH = Math.max(1, bodyBottom - bodyTop);
      ctx.fillRect(x - candleW / 2, bodyTop, candleW, bodyH);
    });
  }

  $effect(() => {
    // Re-run whenever `bars` (or theme, via CSS vars) changes.
    void bars;
    draw();
  });

  $effect(() => {
    if (!containerEl) return;
    const observer = new ResizeObserver(() => draw());
    observer.observe(containerEl);
    return () => observer.disconnect();
  });
</script>

<div class="chart-container" bind:this={containerEl}>
  <canvas bind:this={canvasEl}></canvas>
</div>

<style>
  .chart-container {
    width: 100%;
  }
  canvas {
    display: block;
  }
</style>
