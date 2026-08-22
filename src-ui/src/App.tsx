/**
 * SecureForge — Root Application Component
 *
 * Provides routing between the five main pages:
 * - Dashboard: Drive overview and health monitoring
 * - Sanitizer: Secure erasure wizard (Standard + Expert)
 * - Recovery: Forensic file carving and browsing
 * - Reports: Audit log viewer and PDF export
 * - Plugins: Plugin manager (TOML + Lua)
 * - Expert: Expert-only advanced tools (gated)
 */

function App() {
  return (
    <div className="min-h-screen bg-forge-950 text-white">
      <h1 className="text-2xl font-bold p-8">
        🔒 SecureForge — Sanitize. Recover. Certify.
      </h1>
      {/* TODO: Add React Router and page components */}
    </div>
  );
}

export default App;
