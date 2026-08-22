import { useEffect, useMemo, useRef } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import type { Point } from "./api";

export type ChartSeries = {
  label: string;
  points: Point[];
  dashed?: boolean;
};

const ink = "#18181b";
const muted = "#71717a";
const line = "#e4e4e7";
const barFill = "rgba(24, 24, 27, 0.82)";

export function Chart({
  title,
  series,
  kind = "number",
  mode = "line",
}: {
  title: string;
  series: ChartSeries[];
  kind?: "number" | "ratio" | "percent";
  mode?: "line" | "bars";
}) {
  const root = useRef<HTMLDivElement>(null);
  const readout = useRef<HTMLParagraphElement>(null);
  const signature = useMemo(
    () =>
      `${mode}|${series
        .map((row) => `${row.label}:${row.points.map((point) => `${point.date}=${point.value ?? ""}`).join(",")}`)
        .join("|")}`,
    [mode, series],
  );
  const summary = useMemo(() => describe(title, series, mode), [mode, series, title]);
  const latest = useMemo(() => readoutText(series, lastIndex(series), kind), [kind, series]);

  useEffect(() => {
    const el = root.current;
    if (!el) {
      return;
    }
    const dates = series[0]?.points.map((point) => point.date) ?? [];
    const labels = series[0]?.points.map((point) => point.label || yearLabel(point.date)) ?? [];
    const ys = series.map((row) => row.points.map((point) => point.value));
    const xs = mode === "bars" ? dates.map((_, index) => index) : dates.map((date) => Date.parse(`${date}T00:00:00Z`) / 1000);
    const data: uPlot.AlignedData = [xs, ...ys];
    const write = (index: number | null) => {
      if (readout.current) {
        readout.current.textContent = readoutText(series, index, kind);
      }
    };
    write(lastIndex(series));
    const opts = mode === "bars" ? barOpts(el, labels, kind) : lineOpts(el, series, kind);
    opts.hooks = {
      ...opts.hooks,
      setCursor: [
        (plot) => {
          write(plot.cursor.idx ?? lastIndex(series));
        },
      ],
    };
    const plot = new uPlot(opts, data, el);
    const resize = () => {
      plot.setSize({ width: Math.max(el.clientWidth, 240), height: 180 });
    };
    const observer = new ResizeObserver(resize);
    observer.observe(el);
    return () => {
      observer.disconnect();
      plot.destroy();
    };
    // series is captured via signature so identity churn does not remount the plot
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [kind, signature]);

  return (
    <section className="panel">
      <h2>{title}</h2>
      {series.length > 1 ? (
        <ul className="chart-legend">
          {series.map((row, index) => (
            <li key={row.label} className={index === 0 ? "is-ink" : "is-dash"}>
              {row.label}
            </li>
          ))}
        </ul>
      ) : null}
      <p className="chart-readout" ref={readout}>
        {latest}
      </p>
      <div className="chart" ref={root} role="img" aria-label={summary} />
    </section>
  );
}

function lineOpts(el: HTMLDivElement, series: ChartSeries[], kind: "number" | "ratio" | "percent"): uPlot.Options {
  return {
    width: Math.max(el.clientWidth, 240),
    height: 180,
    padding: [8, 12, 8, 52],
    cursor: { show: true, lock: false },
    legend: { show: false },
    scales: { x: { time: true } },
    axes: [axisX(), axisY(kind)],
    series: [
      {},
      ...series.map((row, index) => ({
        label: row.label,
        stroke: index === 0 ? ink : muted,
        width: index === 0 ? 2 : 1.5,
        dash: row.dashed || index > 0 ? [5, 4] : undefined,
        points: { show: false },
      })),
    ],
  };
}

function barOpts(el: HTMLDivElement, labels: string[], kind: "number" | "ratio" | "percent"): uPlot.Options {
  const bars = uPlot.paths.bars?.({
    size: [0.64, 26, 3],
    align: 0,
    gap: 2,
    radius: [0.08, 0],
  });
  return {
    width: Math.max(el.clientWidth, 240),
    height: 180,
    padding: [8, 12, 8, 52],
    cursor: { show: true, lock: false },
    legend: { show: false },
    scales: {
      x: {
        time: false,
        range: (_self, min, max) => [min - 0.65, max + 0.65],
      },
      y: { range: includeZero },
    },
    axes: [
      {
        ...axisX(),
        grid: { show: false },
        splits: (self, _axis, min, max) => yearSplits(self, labels.length, min, max),
        values: (_self, ticks) => ticks.map((tick) => labels[Math.round(tick)] ?? ""),
      },
      axisY(kind),
    ],
    series: [
      {},
      {
        stroke: ink,
        fill: barFill,
        width: 1,
        points: { show: false },
        paths: bars,
      },
    ],
    hooks: {
      drawAxes: [drawZero],
    },
  };
}

function axisX(): uPlot.Axis {
  return {
    stroke: muted,
    grid: { stroke: line },
    ticks: { stroke: line },
    font: "12px ui-sans-serif, system-ui, sans-serif",
  };
}

function axisY(kind: "number" | "ratio" | "percent"): uPlot.Axis {
  return {
    stroke: muted,
    grid: { stroke: line },
    ticks: { stroke: line },
    font: "12px ui-sans-serif, system-ui, sans-serif",
    values: (_self, ticks) => ticks.map((tick) => formatTick(tick, kind)),
  };
}

function includeZero(_self: uPlot, min: number, max: number): [number, number] {
  if (!Number.isFinite(min) || !Number.isFinite(max)) {
    return [0, 1];
  }
  const lo = Math.min(0, min);
  const hi = Math.max(0, max);
  if (lo === hi) {
    if (lo === 0) {
      return [0, 1];
    }
    return lo > 0 ? [0, lo * 1.1] : [lo * 1.1, 0];
  }
  const pad = (hi - lo) * 0.06;
  return [lo === 0 ? 0 : lo - pad, hi === 0 ? 0 : hi + pad];
}

function yearSplits(self: uPlot, count: number, min: number, max: number): number[] {
  const first = Math.max(0, Math.ceil(min));
  const last = Math.min(count - 1, Math.floor(max));
  if (last < first) {
    return [];
  }
  const labelW = 44;
  const maxTicks = Math.max(2, Math.floor(self.bbox.width / labelW));
  const step = Math.max(1, Math.ceil((last - first) / maxTicks));
  const ticks: number[] = [];
  for (let index = first; index <= last; index += step) {
    ticks.push(index);
  }
  if (ticks[ticks.length - 1] !== last) {
    ticks.push(last);
  }
  return ticks;
}

function yearLabel(date: string | undefined): string {
  return date?.slice(0, 4) ?? "";
}

function lastIndex(series: ChartSeries[]): number | null {
  const points = series[0]?.points ?? [];
  for (let index = points.length - 1; index >= 0; index -= 1) {
    if (points[index]?.value != null) {
      return index;
    }
  }
  return null;
}

function readoutText(series: ChartSeries[], index: number | null, kind: "number" | "ratio" | "percent"): string {
  if (index == null) {
    return "";
  }
  const stamp = series[0]?.points[index]?.label || series[0]?.points[index]?.date || "";
  const values = series
    .map((row) => {
      const value = row.points[index]?.value;
      if (value == null) {
        return null;
      }
      const name = series.length > 1 ? `${row.label} ` : "";
      const change = row.points[index]?.yoy;
      const yoy = series.length === 1 && change != null ? ` · ${formatYoy(change)}` : "";
      return `${name}${formatTick(value, kind)}${yoy}`;
    })
    .filter((part): part is string => part != null);
  return [stamp, ...values].filter(Boolean).join(" · ");
}

function drawZero(self: uPlot) {
  const y0 = self.valToPos(0, "y", true);
  const { left, width } = self.bbox;
  if (!Number.isFinite(y0) || y0 < self.bbox.top || y0 > self.bbox.top + self.bbox.height) {
    return;
  }
  const ctx = self.ctx;
  ctx.save();
  ctx.strokeStyle = ink;
  ctx.globalAlpha = 0.22;
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(left, y0);
  ctx.lineTo(left + width, y0);
  ctx.stroke();
  ctx.restore();
}

function describe(title: string, series: ChartSeries[], mode: "line" | "bars"): string {
  const first = series[0]?.points.find((point) => point.value != null);
  const last = [...(series[0]?.points ?? [])].reverse().find((point) => point.value != null);
  if (!first || last == null) {
    return `${title} chart, no values`;
  }
  const kind = mode === "bars" ? "columns" : "line";
  return `${title} ${kind} from ${first.date} to ${last.date}`;
}

function formatYoy(value: number): string {
  const signed = `${value >= 0 ? "+" : ""}${(Math.round(value * 1000) / 10).toFixed(1)}%`;
  return signed;
}

function formatTick(value: number, kind: "number" | "ratio" | "percent"): string {
  if (!Number.isFinite(value)) {
    return "—";
  }
  if (kind === "percent") {
    return `${Math.round(value * 100)}%`;
  }
  if (kind === "ratio") {
    return value.toFixed(1);
  }
  const abs = Math.abs(value);
  if (abs >= 1e9) {
    return `${(value / 1e9).toFixed(1)}B`;
  }
  if (abs >= 1e6) {
    return `${(value / 1e6).toFixed(1)}M`;
  }
  return String(Math.round(value * 10) / 10);
}
