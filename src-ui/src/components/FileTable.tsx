import { useState } from 'react';
import { CarvedFile } from '../lib/types';
import { FileCode, Search, Download, Eye, FileText, Image, Database, Archive } from 'lucide-react';

interface Props {
  files: CarvedFile[];
  onSelectFile?: (file: CarvedFile) => void;
  selectedFile?: CarvedFile | null;
}

export function FileTable({ files, onSelectFile, selectedFile }: Props) {
  const [searchTerm, setSearchTerm] = useState('');
  const [activeCategory, setActiveCategory] = useState<string>('All');
  const [page, setPage] = useState(0);
  const pageSize = 10;

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  };

  const getCategoryIcon = (category: string) => {
    switch (category.toLowerCase()) {
      case 'media':
      case 'image':
        return <Image className="w-4 h-4 text-cyber-400" />;
      case 'document':
        return <FileText className="w-4 h-4 text-emerald-400" />;
      case 'database':
        return <Database className="w-4 h-4 text-amber-400" />;
      case 'archive':
        return <Archive className="w-4 h-4 text-purple-400" />;
      default:
        return <FileCode className="w-4 h-4 text-slate-400" />;
    }
  };

  const getConfidencePill = (conf: number) => {
    if (conf >= 80) {
      return (
        <span className="px-2 py-0.5 rounded-full text-xs font-semibold bg-emerald-950/80 text-emerald-400 border border-emerald-800/60 font-mono">
          {conf}% High
        </span>
      );
    }
    if (conf >= 50) {
      return (
        <span className="px-2 py-0.5 rounded-full text-xs font-semibold bg-amber-950/80 text-amber-400 border border-amber-800/60 font-mono">
          {conf}% Med
        </span>
      );
    }
    return (
      <span className="px-2 py-0.5 rounded-full text-xs font-semibold bg-rose-950/80 text-rose-400 border border-rose-800/60 font-mono">
        {conf}% Low
      </span>
    );
  };

  const categories = ['All', ...Array.from(new Set(files.map((f) => f.category || 'General')))];

  const filtered = files.filter((f) => {
    const matchesCat = activeCategory === 'All' || (f.category || 'General') === activeCategory;
    const matchesSearch =
      f.filename.toLowerCase().includes(searchTerm.toLowerCase()) ||
      f.file_type.toLowerCase().includes(searchTerm.toLowerCase());
    return matchesCat && matchesSearch;
  });

  const totalPages = Math.ceil(filtered.length / pageSize) || 1;
  const paginated = filtered.slice(page * pageSize, (page + 1) * pageSize);

  return (
    <div className="glass-panel rounded-xl overflow-hidden border border-white/10">
      {/* Search & Category Filter Toolbar */}
      <div className="p-3.5 bg-surface-900/90 border-b border-white/5 flex flex-wrap items-center justify-between gap-3">
        {/* Category Pills */}
        <div className="flex items-center gap-1.5 overflow-x-auto">
          {categories.map((cat) => (
            <button
              key={cat}
              onClick={() => {
                setActiveCategory(cat);
                setPage(0);
              }}
              className={`px-3 py-1 rounded-lg text-xs font-medium transition-all ${
                activeCategory === cat
                  ? 'bg-cyber-500 text-white font-semibold shadow-lg shadow-cyber-500/20'
                  : 'bg-surface-800/60 text-slate-400 hover:text-slate-200 hover:bg-surface-700/60 border border-white/5'
              }`}
            >
              {cat}
            </button>
          ))}
        </div>

        {/* Search Input */}
        <div className="relative w-64">
          <Search className="w-3.5 h-3.5 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
          <input
            type="text"
            placeholder="Search carved files..."
            value={searchTerm}
            onChange={(e) => {
              setSearchTerm(e.target.value);
              setPage(0);
            }}
            className="w-full bg-surface-950/80 border border-white/10 rounded-lg pl-8 pr-3 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:border-cyber-500"
          />
        </div>
      </div>

      {/* Table */}
      <div className="overflow-x-auto">
        <table className="w-full text-left text-xs border-collapse">
          <thead>
            <tr className="bg-surface-950/90 text-slate-400 border-b border-white/5 uppercase tracking-wider font-semibold">
              <th className="py-2.5 px-4">Type / Filename</th>
              <th className="py-2.5 px-4">Offset (LBA)</th>
              <th className="py-2.5 px-4">Size</th>
              <th className="py-2.5 px-4">Confidence</th>
              <th className="py-2.5 px-4 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-white/[0.03]">
            {paginated.length === 0 ? (
              <tr>
                <td colSpan={5} className="py-8 text-center text-slate-500">
                  No carved forensic artifacts found matching filters
                </td>
              </tr>
            ) : (
              paginated.map((file) => (
                <tr
                  key={file.id}
                  onClick={() => onSelectFile && onSelectFile(file)}
                  className={`hover:bg-white/[0.04] transition-colors cursor-pointer ${
                    selectedFile?.id === file.id ? 'bg-cyber-500/10 border-l-2 border-cyber-400' : ''
                  }`}
                >
                  <td className="py-2.5 px-4">
                    <div className="flex items-center gap-2.5">
                      <div className="p-1.5 rounded bg-surface-800 border border-white/5">
                        {getCategoryIcon(file.category || file.file_type)}
                      </div>
                      <div>
                        <span className="font-mono font-medium text-slate-200 block">{file.filename}</span>
                        <span className="text-[10px] text-slate-500 uppercase">{file.file_type}</span>
                      </div>
                    </div>
                  </td>
                  <td className="py-2.5 px-4 font-mono text-cyber-400">
                    0x{(file.offset_bytes || 0).toString(16).toUpperCase()}
                  </td>
                  <td className="py-2.5 px-4 font-mono text-slate-300">
                    {formatBytes(file.size_bytes)}
                  </td>
                  <td className="py-2.5 px-4">{getConfidencePill(file.confidence)}</td>
                  <td className="py-2.5 px-4 text-right">
                    <div className="flex items-center justify-end gap-2" onClick={(e) => e.stopPropagation()}>
                      <button
                        onClick={() => onSelectFile && onSelectFile(file)}
                        className="p-1.5 rounded hover:bg-surface-700 text-slate-400 hover:text-cyber-400 transition-colors"
                        title="Inspect in Hex Viewer"
                      >
                        <Eye className="w-3.5 h-3.5" />
                      </button>
                      <button
                        className="p-1.5 rounded hover:bg-surface-700 text-slate-400 hover:text-emerald-400 transition-colors"
                        title="Export File"
                      >
                        <Download className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination Footer */}
      <div className="px-4 py-2.5 bg-surface-900/90 border-t border-white/5 flex items-center justify-between text-xs text-slate-400">
        <div>
          Showing {paginated.length > 0 ? page * pageSize + 1 : 0} to{' '}
          {Math.min((page + 1) * pageSize, filtered.length)} of {filtered.length} artifacts
        </div>
        <div className="flex items-center gap-2">
          <button
            disabled={page === 0}
            onClick={() => setPage(page - 1)}
            className="px-2.5 py-1 rounded bg-surface-800 disabled:opacity-30 hover:bg-surface-700 text-slate-300 transition-colors"
          >
            Prev
          </button>
          <span className="font-mono text-slate-200">
            {page + 1} / {totalPages}
          </span>
          <button
            disabled={page >= totalPages - 1}
            onClick={() => setPage(page + 1)}
            className="px-2.5 py-1 rounded bg-surface-800 disabled:opacity-30 hover:bg-surface-700 text-slate-300 transition-colors"
          >
            Next
          </button>
        </div>
      </div>
    </div>
  );
}
