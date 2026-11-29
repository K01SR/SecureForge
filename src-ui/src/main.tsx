import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ExpertModeProvider } from "./hooks/ExpertModeContext";
import "./index.css";

class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; error: Error | null }
> {
  state = { hasError: false, error: null as Error | null };

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{ background: '#070c18', color: '#f8fafc', fontFamily: 'system-ui, sans-serif', display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100vh', margin: 0 }}>
          <div style={{ background: '#0f172a', border: '1px solid #e11d48', padding: '2.5rem', borderRadius: '1rem', maxWidth: 500, textAlign: 'center' }}>
            <h1 style={{ color: '#e11d48', marginTop: 0 }}>Application Error</h1>
            <p style={{ color: '#94a3b8', fontSize: '0.85rem' }}>{this.state.error?.message || 'An unexpected error occurred.'}</p>
            <button onClick={() => { this.setState({ hasError: false, error: null }); window.location.reload(); }} style={{ marginTop: '1rem', padding: '0.5rem 1.5rem', background: '#e11d48', color: 'white', border: 'none', borderRadius: '0.5rem', cursor: 'pointer', fontWeight: 'bold' }}>
              Reload Application
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <ExpertModeProvider>
        <App />
      </ExpertModeProvider>
    </ErrorBoundary>
  </React.StrictMode>,
);
