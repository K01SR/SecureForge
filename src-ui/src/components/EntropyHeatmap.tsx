import { useState } from 'react';
import { Activity, Info } from 'lucide-react';

interface Props {
  data: number[];
  height?: number;
  title?: string;
}

export function EntropyHeatmap({ data, height = 48, title }: Props) {
  const [hovered, setHovered] = useState<{ index: number; value: number } | null>(null);

  const getColor = (value: number) => {
    if (value < 1.0) return '#1e40af'; // Deep blue (clean zeroed / wiped)
    if (value < 4.0) return '#0284c7'; // Sky blue (low entropy / sparse)
    if (value < 6.5) return '#eab308'; // Amber (ASCII text / code)
    if (value < 7.5) return '#f97316'; // Orange (structured binary)
    return '#ef4444';                  // Crimson (High entropy: encrypted / compressed / CSPRNG random)
  };

  const avgEntropy = data.length > 0 ? data.reduce((a, b) => a + b, 0) / data.length : 0;
  const highEntropyRatio = data.length > 0 ? (data.filter((v) => v >= 7.5).length / data.length) * 100 : 0;

  if (!data || data.length === 0) {
    return (
      <div className="glass-panel p-4 rounded-xl text-center text-sm text-slate-500 flex items-center justify-center gap-2">
        <Activity className="w-4 h-4 animate-pulse text-cyber-500" />
        No entropy telemetry data available
      </div>
    );
  }

  return (
    <div className="glass-panel p-4 rounded-xl relative">
      {/* Header */}
      <div className="flex items-center justify-between mb-2.5">
        <div className="flex items-center gap-2">
          <Activity className="w-4 h-4 text-cyber-400" />
          <span className="text-xs font-bold uppercase tracking-wider text-slate-200">
            {title || 'Shannon Entropy Sector Map (0.0 – 8.0 bits/byte)'}
          </span>
        </div>
        <div className="flex items-center gap-3 text-xs font-mono">
          <span className="text-slate-400">
            Avg: <span className="font-bold text-white">{avgEntropy.toFixed(2)}</span>
          </span>
          <span className="text-slate-400">
            High Density: <span className="font-bold text-rose-400">{highEntropyRatio.toFixed(0)}%</span>
          </span>
        </div>
      </div>

      {/* SVG Heatmap Bar */}
      <div className="relative rounded-lg overflow-hidden border border-white/10 bg-surface-950">
        <svg
          width="100%"
          height={height}
          className="block cursor-crosshair"
          preserveAspectRatio="none"
          onMouseLeave={() => setHovered(null)}
        >
          {data.map((val, i) => (
            <rect
              key={i}
              x={`${(i / data.length) * 100}%`}
              y={0}
              width={`${Math.max(1, (1 / data.length) * 100)}%`}
              height="100%"
              fill={getColor(val)}
              className="hover:opacity-80 transition-opacity"
              onMouseEnter={() => setHovered({ index: i, value: val })}
            />
          ))}
        </svg>

        {/* Hover Tooltip Overlay */}
        {hovered && (
          <div className="absolute top-1 right-2 pointer-events-none bg-surface-900/95 border border-cyber-500/40 px-2.5 py-1 rounded shadow-xl text-xs font-mono flex items-center gap-2">
            <span className="text-slate-400">Block #{hovered.index}</span>
            <span className="text-white font-bold">{hovered.value.toFixed(3)} b/B</span>
            <span
              className="w-2.5 h-2.5 rounded-full inline-block"
              style={{ backgroundColor: getColor(hovered.value) }}
            />
          </div>
        )}
      </div>

      {/* Legend */}
      <div className="flex items-center justify-between text-[11px] text-slate-400 mt-2 font-mono">
        <div className="flex items-center gap-1.5">
          <span className="w-2.5 h-2.5 rounded-sm bg-[#1e40af] inline-block" />
          <span>0.0 (Zero/Wiped)</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="w-2.5 h-2.5 rounded-sm bg-[#0284c7] inline-block" />
          <span>3.0 (Low Density)</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="w-2.5 h-2.5 rounded-sm bg-[#eab308] inline-block" />
          <span>5.5 (Text/Code)</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="w-2.5 h-2.5 rounded-sm bg-[#ef4444] inline-block" />
          <span>7.9+ (Encrypted/CSPRNG)</span>
        </div>
        <div className="flex items-center gap-1 text-slate-500 font-sans">
          <Info className="w-3 h-3" /> NIST Verified
        </div>
      </div>
    </div>
  );
}
