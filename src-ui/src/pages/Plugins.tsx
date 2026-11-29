import { useState, useEffect } from 'react';
import { PluginsAPI } from '../lib/api';
import { PluginItem } from '../lib/types';
import { Code, FileCode, ShieldCheck, Plus, RefreshCw } from 'lucide-react';

export function Plugins() {
  const [plugins, setPlugins] = useState<PluginItem[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchPlugins = () => {
    setLoading(true);
    PluginsAPI.list()
      .then((data) => {
        setPlugins(data);
        setLoading(false);
      })
      .catch((err) => {
        console.error('Failed to load plugins:', err);
        setLoading(false);
      });
  };

  useEffect(() => {
    fetchPlugins();
  }, []);

  return (
    <div className="space-y-6 pb-12">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 glass-panel p-6 rounded-2xl border border-white/10 bg-gradient-to-r from-surface-900/90 via-surface-900/60 to-surface-950/90">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <span className="px-2.5 py-0.5 rounded-full text-[11px] font-bold uppercase tracking-wider bg-cyber-500/20 text-cyber-400 border border-cyber-500/30">
              Extensible Signature Engine
            </span>
          </div>
          <h1 className="text-2xl font-extrabold tracking-tight text-white">
            TOML Signatures & Sandboxed Lua Host
          </h1>
          <p className="text-xs text-slate-400 mt-1">
            Real filesystem plugins parsed directly from <code>plugins/signatures/</code> and <code>plugins/scripts/</code>.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={fetchPlugins}
            disabled={loading}
            className="flex items-center gap-2 px-3.5 py-2.5 rounded-xl bg-surface-800 hover:bg-surface-700 text-slate-300 text-xs font-bold transition-all border border-white/5 cursor-pointer disabled:opacity-50"
          >
            <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} /> Reload Plugins
          </button>
          <button className="flex items-center gap-2 px-4 py-2.5 rounded-xl bg-gradient-to-r from-cyber-600 to-cyber-500 hover:from-cyber-500 hover:to-cyber-400 text-white font-bold text-xs uppercase tracking-wider shadow-lg shadow-cyber-600/30 transition-all cursor-pointer">
            <Plus className="w-4 h-4" /> Load Custom Plugin
          </button>
        </div>
      </div>

      {/* Grid of Plugins */}
      {loading ? (
        <div className="glass-panel p-12 rounded-xl text-center text-xs text-slate-500 flex items-center justify-center gap-2">
          <RefreshCw className="w-4 h-4 animate-spin text-cyber-400" />
          Loading active signature definitions from disk...
        </div>
      ) : plugins.length === 0 ? (
        <div className="glass-panel p-12 rounded-xl text-center text-xs text-slate-500">
          No active plugins found in <code>plugins/</code>.
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {plugins.map((p) => (
            <div
              key={p.name}
              className="glass-panel p-5 rounded-2xl border border-white/10 hover:border-cyber-500/40 transition-all space-y-3"
            >
              <div className="flex items-start justify-between">
                <div className="p-2.5 rounded-xl bg-surface-950 border border-white/5 text-cyber-400">
                  {p.plugin_type === 'Lua' ? (
                    <Code className="w-5 h-5 text-purple-400" />
                  ) : (
                    <FileCode className="w-5 h-5 text-cyber-400" />
                  )}
                </div>
                <span className="px-2 py-0.5 rounded text-[10px] font-mono font-bold uppercase bg-surface-950 text-slate-300 border border-white/10">
                  {p.plugin_type} Module
                </span>
              </div>

              <div>
                <h3 className="font-bold text-sm text-white">{p.name}</h3>
                <span className="text-xs text-cyber-400 font-mono block mt-0.5">{p.extension}</span>
                <p className="text-xs text-slate-400 mt-2 leading-relaxed">{p.description}</p>
              </div>

              <div className="pt-3 border-t border-white/5 flex items-center justify-between text-xs font-mono">
                <span className="text-slate-500">
                  Category: <strong className="text-slate-300">{p.category}</strong>
                </span>
                <span className="text-emerald-400 flex items-center gap-1">
                  <ShieldCheck className="w-3.5 h-3.5" /> {p.status}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default Plugins;
