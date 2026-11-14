import { useDrives } from '../hooks/useDrives';
import { DriveCard } from '../components/DriveCard';
import { EntropyHeatmap } from '../components/EntropyHeatmap';

export function Dashboard() {
  const { drives, loading, refresh } = useDrives();

  // Mock data for preview
  const previewData = Array.from({length: 100}, () => Math.random() * 8);

  return (
    <div className="p-6 space-y-8 animate-fade-in">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold text-white">System Dashboard</h1>
        <button onClick={refresh} className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded font-medium transition-colors">
          Refresh Disks
        </button>
      </div>

      <section>
        <h2 className="text-xl font-semibold text-gray-300 mb-4">Detected Drives</h2>
        {loading ? (
          <div className="text-gray-400">Loading drives...</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {drives.map((d, i) => <DriveCard key={i} drive={d} />)}
            {drives.length === 0 && <div className="text-gray-500">No drives detected</div>}
          </div>
        )}
      </section>

      <section className="bg-gray-800 p-6 rounded-lg border border-gray-700">
        <h2 className="text-xl font-semibold text-gray-300 mb-4">Live System Entropy Preview (sda1)</h2>
        <EntropyHeatmap data={previewData} />
      </section>
    </div>
  );
}
