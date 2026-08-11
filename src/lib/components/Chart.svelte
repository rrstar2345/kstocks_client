<script lang="ts">
  // Candlestick chart built on lightweight-charts (TradingView's open-source
  // charting library): gives us axes, crosshair/tooltip, pan/zoom, and a
  // real API surface for adding indicator overlays later, instead of the
  // old hand-rolled <canvas> renderer.
  import { onMount } from "svelte";
  import {
    createChart,
    CandlestickSeries,
    type IChartApi,
    type ISeriesApi,
    type UTCTimestamp,
  } from "lightweight-charts";
  import type { OhlcBar } from "$lib/types";

  let { bars, height = 320 }: { bars: OhlcBar[]; height?: number } = $props();

  let containerEl: HTMLDivElement;
  let chart: IChartApi | undefined;
  let series: ISeriesApi<"Candlestick"> | undefined;

  function readColor(varName: string): string {
    return getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
  }

  function toUtcTimestamp(bucketStart: string): UTCTimestamp {
    return (Date.parse(bucketStart) / 1000) as UTCTimestamp;
  }

  function applyTheme() {
    if (!chart || !series) return;
    const text = readColor("--color-text-muted");
    const border = readColor("--color-border");
    const positive = readColor("--color-positive");
    const negative = readColor("--color-negative");

    chart.applyOptions({
      layout: {
        background: { color: "transparent" },
        textColor: text,
      },
      grid: {
        vertLines: { color: border },
        horzLines: { color: border },
      },
      timeScale: { borderColor: border },
      rightPriceScale: { borderColor: border },
    });

    series.applyOptions({
      upColor: positive,
      downColor: negative,
      borderUpColor: positive,
      borderDownColor: negative,
      wickUpColor: positive,
      wickDownColor: negative,
    });
  }

  function setData() {
    if (!series) return;

    // Defensive dedupe/sort: lightweight-charts throws if points aren't
    // strictly ascending by time. Bars should already come sorted and
    // unique from the backend, but two bars can legitimately collapse to
    // the same second here if `bucket_start` values are ever
    // sub-minute-precision (or duplicated across a fetch/tick race) — keep
    // the later value for any duplicate timestamp rather than crashing.
    const byTime = new Map<UTCTimestamp, { time: UTCTimestamp; open: number; high: number; low: number; close: number }>();
    for (const b of bars) {
      const time = toUtcTimestamp(b.bucket_start);
      byTime.set(time, { time, open: b.open, high: b.high, low: b.low, close: b.close });
    }
    const points = [...byTime.values()].sort((a, b) => a.time - b.time);

    series.setData(points);
  }

  onMount(() => {
    chart = createChart(containerEl, {
      height,
      autoSize: false,
      timeScale: { timeVisible: true, secondsVisible: false },
      crosshair: { mode: 0 },
    });
    series = chart.addSeries(CandlestickSeries);

    applyTheme();
    setData();

    const resizeObserver = new ResizeObserver((entries) => {
      const width = entries[0]?.contentRect.width;
      if (width && chart) chart.applyOptions({ width });
    });
    resizeObserver.observe(containerEl);

    // Theme toggling flips `data-theme` on <html>; re-read CSS vars when
    // that happens so the chart follows light/dark switches live.
    const themeObserver = new MutationObserver(applyTheme);
    themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });

    return () => {
      resizeObserver.disconnect();
      themeObserver.disconnect();
      chart?.remove();
    };
  });

  $effect(() => {
    void bars;
    setData();
  });
</script>

<div class="chart-container" bind:this={containerEl}></div>

<style>
  .chart-container {
    width: 100%;
    min-width: 0;
    height: auto;
  }
</style>