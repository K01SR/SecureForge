interface Props {
  data: number[];
  width?: number;
  height?: number;
}

export function EntropyHeatmap({ data, height = 40 }: Props) {
  const getColor = (value: number) => {
    if (value < 2.0) return 'rgb(37, 99, 235)'; // Blue - likely wiped
    if (value < 7.0) return 'rgb(234, 179, 8)'; // Yellow - mixed/text
    return 'rgb(220, 38, 38)'; // Red - compressed/encrypted/high entropy
  };

  if (!data || data.length === 0) {
    return <div className="h-10 w-full bg-gray-800 flex items-center justify-center text-sm text-gray-500 rounded">No data</div>;
  }

  return (
    <div className="relative group">
      <svg width="100%" height={height} className="rounded border border-gray-700 bg-gray-900 block" preserveAspectRatio="none">
        {data.map((val, i) => (
          <rect
            key={i}
            x={`${(i / data.length) * 100}%`}
            y={0}
            width={`${(1 / data.length) * 100}%`}
            height="100%"
            fill={getColor(val)}
            className="hover:opacity-75 transition-opacity cursor-crosshair"
          >
            <title>{`Offset: ${i}, Entropy: ${val.toFixed(2)}`}</title>
          </rect>
        ))}
      </svg>
      <div className="flex justify-between text-xs text-gray-400 mt-1">
        <span>0</span>
        <span>Low (Wiped)</span>
        <span>High (Data)</span>
        <span>{data.length}</span>
      </div>
    </div>
  );
}
