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

  // `bucket_start` from the backend is a genuine UTC instant (see
  // src-tauri/src/storage/ohlc.rs: bars are floored from `Utc::now()`
  // tick-arrival timestamps and stringified with a trailing `Z`), so
  // `toUtcTimestamp` above is correct as-is — no manual IST shift needed
  // there. What *was* wrong is display: lightweight-charts renders axis/
  // crosshair labels using the *browser's* local timezone by default. On
  // any machine not set to IST, that silently relabels NSE bars into the
  // viewer's local time instead of the market's actual IST wall-clock
  // time. Since NSE market hours and trader expectations are IST, format
  // every label as fixed UTC+5:30 explicitly rather than relying on
  // (or fighting) the browser's local offset.
  const IST_OFFSET_MS = (5 * 60 + 30) * 60 * 1000;

  function toIstDate(unixSeconds: number): Date {
    // A Date whose UTC getters read back IST wall-clock fields, by
    // shifting the instant forward by the fixed IST offset before
    // handing it to Date. This avoids any dependence on the host's
    // configured timezone/locale.
    return new Date(unixSeconds * 1000 + IST_OFFSET_MS);
  }

  function pad2(n: number): string {
    return n < 10 ? `0${n}` : `${n}`;
  }

  function formatIstTime(unixSeconds: number): string {
    const d = toIstDate(unixSeconds);
    return `${pad2(d.getUTCHours())}:${pad2(d.getUTCMinutes())}`;
  }

  function formatIstDate(unixSeconds: number): string {
    const d = toIstDate(unixSeconds);
    const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    return `${d.getUTCDate()} ${months[d.getUTCMonth()]} '${String(d.getUTCFullYear()).slice(-2)}`;
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
    chart?.timeScale().fitContent();
  }

  onMount(() => {
    // Measure synchronously at creation time instead of waiting for the
    // first ResizeObserver callback — otherwise the chart is created at
    // its default (often 0) width, which can leave the time axis with no
    // room to lay out any tick labels until the first resize fires.
    const initialWidth = containerEl.clientWidth || undefined;

    chart = createChart(containerEl, {
      height,
      width: initialWidth,
      autoSize: false,
      timeScale: {
        timeVisible: true,
        secondsVisible: false,
        borderVisible: true,
        // Labels are formatted explicitly in fixed IST (UTC+5:30) below,
        // independent of the viewer's machine/browser timezone — see the
        // comment above `IST_OFFSET_MS`.
        tickMarkFormatter: (time: UTCTimestamp) => formatIstTime(time as number),
      },
      localization: {
        timeFormatter: (time: UTCTimestamp) => {
          const t = time as number;
          return `${formatIstDate(t)} ${formatIstTime(t)} IST`;
        },
      },
      crosshair: { mode: 0 },
    });
    series = chart.addSeries(CandlestickSeries);

    applyTheme();
    setData();
    chart.timeScale().fitContent();

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
    flex: 1;
    min-height: 0;
  }
</style>