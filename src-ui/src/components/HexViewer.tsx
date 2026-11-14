
interface Props {
  data: Uint8Array;
  offset?: number;
}

export function HexViewer({ data, offset = 0 }: Props) {
  const rows = [];
  for (let i = 0; i < data.length; i += 16) {
    const chunk = data.slice(i, i + 16);
    
    let hex = '';
    let ascii = '';
    
    for (let j = 0; j < 16; j++) {
      if (j < chunk.length) {
        hex += chunk[j].toString(16).padStart(2, '0') + ' ';
        const charCode = chunk[j];
        ascii += (charCode >= 32 && charCode <= 126) ? String.fromCharCode(charCode) : '.';
      } else {
        hex += '   ';
        ascii += ' ';
      }
    }
    
    const rowOffset = (offset + i).toString(16).padStart(8, '0');
    rows.push({ offset: rowOffset, hex, ascii });
  }

  return (
    <div className="bg-gray-900 border border-gray-700 rounded p-4 font-mono text-sm overflow-x-auto">
      <table className="w-full text-gray-300">
        <tbody>
          {rows.map((row, i) => (
            <tr key={i} className="hover:bg-gray-800">
              <td className="text-gray-500 pr-4 select-none">{row.offset}</td>
              <td className="pr-4 whitespace-pre text-blue-300">{row.hex}</td>
              <td className="whitespace-pre text-green-400">{row.ascii}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
