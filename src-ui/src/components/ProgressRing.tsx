interface Props {
  percent: number;
  size?: number;
  strokeWidth?: number;
  color?: string;
  speedMbps?: number;
  etaSeconds?: number;
  phase?: string;
}

export function ProgressRing({
  percent,
  size = 180,
  strokeWidth = 10,
  color = '#0ea5e9',
  speedMbps,
  etaSeconds,
  phase,
}: Props) {
  const radius = (size - strokeWidth) / 2;
  const circumference = radius * 2 * Math.PI;
  const offset = circumference - (Math.min(100, Math.max(0, percent)) / 100) * circumference;

  const formatEta = (seconds?: number) => {
    if (seconds === undefined || seconds === null || seconds <= 0) return '00:00';
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div className="flex flex-col items-center justify-center relative">
      <div className="relative flex items-center justify-center" style={{ width: size, height: size }}>
        <svg className="transform -rotate-90" width={size} height={size}>
          {/* Background circle */}
          <circle
            cx={size / 2}
            cy={size / 2}
            r={radius}
            stroke="rgba(255, 255, 255, 0.05)"
            strokeWidth={strokeWidth}
            fill="transparent"
          />
          {/* Progress circle */}
          <circle
            cx={size / 2}
            cy={size / 2}
            r={radius}
            stroke={color}
            strokeWidth={strokeWidth}
            strokeDasharray={circumference}
            strokeDashoffset={offset}
            strokeLinecap="round"
            fill="transparent"
            className="transition-all duration-300 ease-out"
            style={{ filter: `drop-shadow(0 0 8px ${color}80)` }}
          />
        </svg>

        {/* Center Readout */}
        <div className="absolute inset-0 flex flex-col items-center justify-center text-center">
          <span className="text-3xl font-extrabold font-mono tracking-tight text-white">
            {percent.toFixed(1)}%
          </span>
          {phase && (
            <span className="text-xs font-semibold uppercase tracking-wider text-cyber-400 mt-1 px-2 py-0.5 rounded bg-cyber-950/80 border border-cyber-800/40">
              {phase}
            </span>
          )}
        </div>
      </div>

      {/* Speed & ETA metrics */}
      {(speedMbps !== undefined || etaSeconds !== undefined) && (
        <div className="grid grid-cols-2 gap-4 mt-4 w-full max-w-[220px] text-center">
          {speedMbps !== undefined && (
            <div className="bg-surface-900/80 border border-white/5 rounded-lg p-2">
              <span className="text-[10px] text-slate-500 uppercase font-semibold block">Throughput</span>
              <span className="text-sm font-mono font-bold text-emerald-400">{speedMbps.toFixed(1)} MB/s</span>
            </div>
          )}
          {etaSeconds !== undefined && (
            <div className="bg-surface-900/80 border border-white/5 rounded-lg p-2">
              <span className="text-[10px] text-slate-500 uppercase font-semibold block">Est. Time</span>
              <span className="text-sm font-mono font-bold text-cyber-400">{formatEta(etaSeconds)}</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
