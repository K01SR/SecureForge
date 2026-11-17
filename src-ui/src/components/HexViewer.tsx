import { useState } from 'react';
import { Copy, Check, Terminal } from 'lucide-react';

interface Props {
  hexData?: string;
  rawBytes?: Uint8Array;
  startOffset?: number;
  title?: string;
}

export function HexViewer({ hexData, rawBytes, startOffset = 0, title }: Props) {
  const [copied, setCopied] = useState(false);

  // Parse bytes either from raw Uint8Array or hex string
  let bytes: number[] = [];
  if (rawBytes) {
    bytes = Array.from(rawBytes);
  } else if (hexData) {
    const clean = hexData.replace(/\s+/g, '');
    for (let i = 0; i < clean.length; i += 2) {
      bytes.push(parseInt(clean.substring(i, i + 2), 16) || 0);
    }
  }

  // Generate 16-byte rows
  const rows: { offset: string; hexLeft: string; hexRight: string; ascii: string }[] = [];
  for (let i = 0; i < bytes.length; i += 16) {
    const chunk = bytes.slice(i, i + 16);
    const offsetStr = (startOffset + i).toString(16).padStart(8, '0').toUpperCase();

    const hexParts = chunk.map((b) => b.toString(16).padStart(2, '0').toUpperCase());
    while (hexParts.length < 16) {
      hexParts.push('  ');
    }

    const asciiParts = chunk.map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : '.')).join('');

    rows.push({
      offset: offsetStr,
      hexLeft: hexParts.slice(0, 8).join(' '),
      hexRight: hexParts.slice(8, 16).join(' '),
      ascii: asciiParts,
    });
  }

  const handleCopy = () => {
    if (hexData) {
      navigator.clipboard.writeText(hexData);
    } else {
      const full = rows.map((r) => `${r.offset}  ${r.hexLeft}  ${r.hexRight}  |${r.ascii}|`).join('\n');
      navigator.clipboard.writeText(full);
    }
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="glass-panel rounded-xl overflow-hidden border border-white/10 font-mono text-xs">
      {/* Header bar */}
      <div className="bg-surface-900/90 px-4 py-2.5 border-b border-white/5 flex items-center justify-between">
        <div className="flex items-center gap-2 text-slate-300">
          <Terminal className="w-4 h-4 text-cyber-400" />
          <span className="font-sans font-semibold text-xs tracking-wide">
            {title || 'Forensic Hex & ASCII Sector Inspector'}
          </span>
          <span className="text-[10px] text-slate-500 bg-surface-950 px-2 py-0.5 rounded border border-white/5">
            {bytes.length} bytes loaded
          </span>
        </div>
        <button
          onClick={handleCopy}
          className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-surface-800 hover:bg-surface-700 text-slate-300 hover:text-white transition-colors border border-white/5 text-xs font-sans"
        >
          {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
          {copied ? 'Copied' : 'Copy Dump'}
        </button>
      </div>

      {/* Hex Dump Container */}
      <div className="p-4 bg-surface-950/95 overflow-x-auto max-h-80 select-text">
        {rows.length === 0 ? (
          <div className="text-slate-500 py-6 text-center font-sans">No byte payload to display</div>
        ) : (
          <table className="w-full text-left border-collapse leading-relaxed">
            <thead>
              <tr className="text-slate-500 border-b border-white/5 pb-1 select-none">
                <th className="w-24 text-cyber-500/80 font-normal">Offset</th>
                <th className="text-slate-400 font-normal tracking-widest pl-4">00 01 02 03 04 05 06 07</th>
                <th className="text-slate-400 font-normal tracking-widest pl-4">08 09 0A 0B 0C 0D 0E 0F</th>
                <th className="text-slate-400 font-normal pl-6">Decoded Text</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/[0.02]">
              {rows.map((row, idx) => (
                <tr key={idx} className="hover:bg-white/[0.03] transition-colors">
                  <td className="text-cyber-400 font-bold select-none">{row.offset}</td>
                  <td className="text-slate-200 tracking-wider pl-4">{row.hexLeft}</td>
                  <td className="text-slate-200 tracking-wider pl-4">{row.hexRight}</td>
                  <td className="text-emerald-400/90 pl-6 border-l border-white/5 tracking-normal">
                    {row.ascii}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
