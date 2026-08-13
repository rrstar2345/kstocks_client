//! First-pass POC: candlestick chart in egui + egui_plot, redrawing the
//! last (in-progress) candle continuously at 30Hz as synthetic ticks
//! arrive. Goal: prove out rendering latency/CPU characteristics before
//! porting the real Svelte/lightweight-charts chart over.
//!
//! Not wired to the Tauri backend yet — ticks here are generated locally
//! with a simple random walk so this binary is fully standalone and can
//! be run with `cargo run --release`.

use std::time::{Duration, Instant};

use eframe::egui;
use egui_plot::{BoxElem, BoxPlot, BoxSpread, HPlacement, Plot, PlotPoint};

/// One OHLC bar. Mirrors the shape of `OhlcBar` from the Tauri backend
/// (`src-tauri/src/storage/ohlc.rs`) closely enough to swap in real data
/// later: open/high/low/close plus a bucket start time.
#[derive(Clone, Copy, Debug)]
struct Candle {
    /// Bucket index, used as the plot's x-argument (seconds since start).
    bucket: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

impl Candle {
    fn new(bucket: i64, price: f64) -> Self {
        Self {
            bucket,
            open: price,
            high: price,
            low: price,
            close: price,
        }
    }

    fn apply_tick(&mut self, price: f64) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
    }
}

/// Growing history of candles, oldest first. Unbounded here on purpose —
/// older bars stay in the series and simply scroll off-screen (pan/zoom
/// to see them) rather than being evicted. The real app will eventually
/// want to page older history in from `get_recent_index_bars` instead of
/// holding everything in memory forever, but that's a data-loading
/// concern, not a rendering one, so it's out of scope for this POC.
struct CandleSeries {
    bars: Vec<Candle>,
    bucket_seconds: i64,
}

impl CandleSeries {
    fn new(bucket_seconds: i64) -> Self {
        Self {
            bars: Vec::new(),
            bucket_seconds,
        }
    }

    /// Feed one tick. Rolls the last candle into a new one when the tick's
    /// bucket has advanced; otherwise updates the last candle in place.
    fn push_tick(&mut self, elapsed_secs: f64, price: f64) {
        let bucket = (elapsed_secs as i64) / self.bucket_seconds;
        match self.bars.last_mut() {
            Some(last) if last.bucket == bucket => {
                last.apply_tick(price);
            }
            _ => {
                self.bars.push(Candle::new(bucket, price));
            }
        }
    }
}

/// Synthetic tick source: a simple bounded random walk so the demo is
/// self-contained. Swap this out for the real websocket/tick pipeline
/// (`src-tauri/src/market/streamers`) when wiring up live data.
struct TickGenerator {
    price: f64,
    rng_state: u64,
}

impl TickGenerator {
    fn new(start_price: f64) -> Self {
        Self {
            price: start_price,
            rng_state: 0x2545F4914F6CDD1D,
        }
    }

    fn next_f64(&mut self) -> f64 {
        // xorshift64 — good enough for demo noise, no external dep needed.
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        (x >> 11) as f64 / (1u64 << 53) as f64
    }

    fn next_price(&mut self) -> f64 {
        let step = (self.next_f64() - 0.5) * 2.0;
        self.price = (self.price + step).max(1.0);
        self.price
    }
}

struct CandlestickApp {
    series: CandleSeries,
    generator: TickGenerator,
    start: Instant,
    last_tick_at: Instant,
    tick_interval: Duration,
    frame_times: Vec<f32>,
    last_frame_at: Instant,
    tick_count: u64,
}

impl CandlestickApp {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            series: CandleSeries::new(1), // 1-second candles, unbounded history
            generator: TickGenerator::new(100.0),
            start: now,
            last_tick_at: now,
            // Simulate a moderately busy tick feed independent of the
            // 30Hz redraw rate, e.g. every 20ms.
            tick_interval: Duration::from_millis(20),
            frame_times: Vec::with_capacity(240),
            last_frame_at: now,
            tick_count: 0,
        }
    }
}

impl eframe::App for CandlestickApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // --- drive the synthetic tick feed ---
        let now = Instant::now();
        while now.duration_since(self.last_tick_at) >= self.tick_interval {
            self.last_tick_at += self.tick_interval;
            let elapsed = self.last_tick_at.duration_since(self.start).as_secs_f64();
            let price = self.generator.next_price();
            self.series.push_tick(elapsed, price);
            self.tick_count += 1;
        }

        // --- frame pacing bookkeeping for the on-screen readout ---
        let dt = now.duration_since(self.last_frame_at).as_secs_f32();
        self.last_frame_at = now;
        if dt > 0.0 {
            self.frame_times.push(dt);
            if self.frame_times.len() > 120 {
                self.frame_times.remove(0);
            }
        }
        let avg_dt = if self.frame_times.is_empty() {
            0.0
        } else {
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
        };
        let fps = if avg_dt > 0.0 { 1.0 / avg_dt } else { 0.0 };

        ui.heading("egui + egui_plot candlestick — 30Hz redraw POC");
        ui.horizontal(|ui| {
            ui.label(format!("bars: {}", self.series.bars.len()));
            ui.separator();
            ui.label(format!("ticks received: {}", self.tick_count));
            ui.separator();
            ui.label(format!("redraw fps: {fps:.1}"));
            ui.separator();
            ui.label(format!("frame time: {:.2} ms", avg_dt * 1000.0));
        });
        ui.separator();

        let up_color = egui::Color32::from_rgb(38, 166, 154);
        let down_color = egui::Color32::from_rgb(239, 83, 80);

        // Stash open/high/low/close alongside each box so the tooltip
        // formatter (which only receives the BoxElem/BoxSpread, not our
        // Candle type) can still print real OHLC instead of
        // quartile/median language. BoxSpread's fields already *are*
        // O/H/L/C under the hood (see mapping below), so we just need to
        // relabel them in the formatter rather than carry extra state.
        //
        // Mapping used when building each box:
        //   lower_whisker = low
        //   quartile1     = min(open, close)   (bottom of body)
        //   median        = close
        //   quartile3     = max(open, close)   (top of body)
        //   upper_whisker = high
        // To recover open/close for the tooltip we also need to know
        // which of quartile1/quartile3 was the open vs the close — encode
        // that with the box's fill color (up_color => open=lower body,
        // down_color => open=upper body), which the formatter checks.
        let boxes: Vec<BoxElem> = self
            .series
            .bars
            .iter()
            .map(|c| {
                let is_up = c.close >= c.open;
                let (fill, stroke_color) = if is_up { (up_color, up_color) } else { (down_color, down_color) };
                let body_top = c.open.max(c.close);
                let body_bottom = c.open.min(c.close);
                BoxElem::new(
                    c.bucket as f64,
                    BoxSpread::new(c.low, body_bottom, c.close, body_top, c.high),
                )
                .name(format!("{:.2}|{:.2}|{is_up}", c.open, c.close))
                .box_width(0.7)
                .whisker_width(0.0)
                .fill(fill)
                .stroke(egui::Stroke::new(1.0, stroke_color))
            })
            .collect();

        let box_plot = BoxPlot::new("candles", boxes).vertical().element_formatter(Box::new(
            |elem: &egui_plot::BoxElem, _plot: &BoxPlot| {
                // Parse back the open/close/is_up we stashed in `name`
                // (see comment above) rather than reinterpreting
                // quartile1/quartile3, so this reads correctly regardless
                // of candle direction.
                let mut parts = elem.name.splitn(3, '|');
                let open: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(elem.spread.quartile1);
                let close: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(elem.spread.median);
                format!(
                    "time: {}s\nO: {:.2}\nH: {:.2}\nL: {:.2}\nC: {:.2}",
                    elem.argument as i64,
                    open,
                    elem.spread.upper_whisker,
                    elem.spread.lower_whisker,
                    close,
                )
            },
        ));

        let last_close = self.series.bars.last().map(|c| c.close);
        let last_bucket = self.series.bars.last().map(|c| c.bucket as f64);
        let is_last_up = self
            .series
            .bars
            .last()
            .map(|c| c.close >= c.open)
            .unwrap_or(true);
        let last_color = if is_last_up { up_color } else { down_color };

        let plot_response = Plot::new("candlestick_plot")
            .height(ui.available_height() - 8.0)
            .show_grid(true)
            .allow_scroll(true)
            .allow_zoom(true)
            // Price axis on the right, like TradingView.
            .y_axis_position(HPlacement::Right)
            .x_axis_formatter(|mark, _range| format!("{}s", mark.value as i64))
            .show(ui, |plot_ui| {
                plot_ui.box_plot(box_plot);

                // Dotted horizontal line at the last traded price,
                // spanning from the last candle to the right edge of the
                // visible plot area.
                if let (Some(price), Some(bucket)) = (last_close, last_bucket) {
                    let x_max = plot_ui.plot_bounds().max()[0];
                    let line = egui_plot::Line::new(
                        "last_price",
                        vec![[bucket, price], [x_max, price]],
                    )
                    .color(last_color)
                    .style(egui_plot::LineStyle::Dotted { spacing: 4.0 })
                    .width(1.0);
                    plot_ui.line(line);
                }
            });

        // Price tag box at the right edge of the plot, at the last
        // traded price — painted in screen space using the transform
        // egui_plot hands back, since egui_plot itself has no built-in
        // "axis price tag" widget.
        if let Some(price) = last_close {
            let transform = &plot_response.transform;
            let plot_rect = *transform.frame();
            let y = transform.position_from_point(&PlotPoint::new(0.0, price)).y;

            let label = format!("{price:.2}");
            let painter = ui.painter_at(plot_rect.expand2(egui::vec2(60.0, 0.0)));
            let font = egui::FontId::proportional(12.0);
            let galley = painter.layout_no_wrap(label.clone(), font.clone(), egui::Color32::WHITE);
            let padding = egui::vec2(6.0, 3.0);
            let tag_size = galley.size() + padding * 2.0;
            let tag_min = egui::pos2(plot_rect.right(), y - tag_size.y / 2.0);
            let tag_rect = egui::Rect::from_min_size(tag_min, tag_size);

            painter.rect_filled(tag_rect, egui::CornerRadius::same(2), last_color);
            painter.text(
                tag_rect.left_center() + egui::vec2(padding.x, 0.0),
                egui::Align2::LEFT_CENTER,
                label,
                font,
                egui::Color32::WHITE,
            );
        }

        // Force continuous repaint so the last candle animates smoothly
        // at ~30Hz regardless of input events. This is the core thing
        // this POC is meant to validate: does egui hold 30fps here with
        // low, consistent frame time.
        ctx.request_repaint_after(Duration::from_millis(33));
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "kstocks candlestick POC",
        native_options,
        Box::new(|_cc| Ok(Box::new(CandlestickApp::new()))),
    )
}