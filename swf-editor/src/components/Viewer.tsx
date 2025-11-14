import React from 'react';
import { ResourceType, ResourceInfo } from '../types';
import { invoke } from '@tauri-apps/api/core';

interface ViewerProps {
  resourceType: ResourceType;
  resourceId: number;
  resourceInfo: ResourceInfo;
}

export const Viewer: React.FC<ViewerProps> = ({ resourceType, resourceId, resourceInfo }) => {
  const [data, setData] = React.useState<Uint8Array | null>(null);
  const [decompiled, setDecompiled] = React.useState<string | null>(null);
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    setLoading(true);
    setError(null);
    setData(null);
    setDecompiled(null);

    const loadResource = async () => {
      try {
        if (resourceType === 'scripts') {
          // For scripts, decompile them
          const code = await invoke<string>('decompile_script', {
            scriptId: resourceId,
          });
          setDecompiled(code);
        } else {
          // For other resources, get raw data
          const rawData = await invoke<number[]>('get_resource_data', {
            resourceType: resourceType.slice(0, -1), // Remove 's' from plural
            resourceId: resourceId,
          });
          setData(new Uint8Array(rawData));
        }
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    };

    loadResource();
  }, [resourceType, resourceId]);

  const handleExport = async () => {
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const filePath = await save({
        defaultPath: `${resourceInfo.name || `resource_${resourceId}`}`,
      });

      if (filePath) {
        await invoke('export_resource', {
          resourceType: resourceType.slice(0, -1),
          resourceId: resourceId,
          outputPath: filePath,
        });
        alert('Resource exported successfully!');
      }
    } catch (err) {
      alert(`Export failed: ${err}`);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="flex items-center gap-2">
          <div className="spinner"></div>
          <span>Loading resource...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4">
        <div className="panel">
          <div className="panel-header text-error">
            <span>Error Loading Resource</span>
          </div>
          <div className="panel-content">
            <p className="text-error">{error}</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="p-4 h-full flex flex-col">
      <div style={{ marginBottom: '16px' }}>
        <h2>{resourceInfo.name || `${resourceType}_${resourceId}`}</h2>
        <div className="panel" style={{ marginTop: '12px' }}>
          <div className="panel-content">
            <div className="flex gap-3" style={{ flexWrap: 'wrap' }}>
              <div><strong className="text-accent">ID:</strong> <span className="text-secondary">{resourceId}</span></div>
              <div><strong className="text-accent">Type:</strong> <span className="text-secondary">{resourceInfo.resource_type}</span></div>
              <div><strong className="text-accent">Size:</strong> <span className="text-secondary">{(resourceInfo.size / 1024).toFixed(2)} KB</span></div>
              {resourceInfo.metadata && Object.entries(resourceInfo.metadata).map(([key, value]) => (
                <div key={key}><strong className="text-accent">{key}:</strong> <span className="text-secondary">{value}</span></div>
              ))}
            </div>
          </div>
        </div>
        <button
          onClick={handleExport}
          className="btn-primary"
          style={{ marginTop: '12px' }}
        >
          📤 Export
        </button>
      </div>

      <div className="panel overflow-auto" style={{ flex: 1 }}>
        {resourceType === 'scripts' && decompiled && (
          <div className="panel-content">
            <pre className="code-editor" style={{ margin: 0, whiteSpace: 'pre-wrap' }}>
              {decompiled}
            </pre>
          </div>
        )}

        {resourceType === 'images' && data && (
          <div className="panel-content" style={{ textAlign: 'center' }}>
            <img
              src={`data:image/png;base64,${btoa(String.fromCharCode(...data))}`}
              alt={`Image ${resourceId}`}
              style={{ maxWidth: '100%', maxHeight: '600px', borderRadius: '4px' }}
              onError={() => {
                setError('Failed to load image');
              }}
            />
          </div>
        )}

        {resourceType === 'sounds' && data && (
          <div className="panel-content">
            <div className="flex items-center gap-2 p-3 rounded" style={{ background: 'var(--bg-tertiary)' }}>
              <span style={{ fontSize: '24px' }}>🔊</span>
              <div>
                <p className="text-secondary" style={{ margin: 0 }}>Sound resource (playback not yet implemented)</p>
                <p className="text-muted" style={{ margin: '4px 0 0 0', fontSize: '12px' }}>Raw data size: {data.length} bytes</p>
              </div>
            </div>
          </div>
        )}

        {resourceType === 'sprites' && (
          <div className="panel-content">
            <div className="flex items-center gap-2 p-3 rounded" style={{ background: 'var(--bg-tertiary)' }}>
              <span style={{ fontSize: '24px' }}>🎬</span>
              <div>
                <p className="text-secondary" style={{ margin: 0 }}>Sprite preview not yet implemented</p>
              </div>
            </div>
          </div>
        )}

        {!resourceType.match(/scripts|images|sounds|sprites/) && data && (
          <div className="panel-content">
            <p className="text-secondary">Binary data ({data.length} bytes)</p>
            <pre className="code-editor" style={{ fontSize: '11px', marginTop: '12px' }}>
              {Array.from(data.slice(0, 256)).map((b, i) =>
                (i % 16 === 0 ? '\n' : '') + b.toString(16).padStart(2, '0') + ' '
              )}
              {data.length > 256 && '\n... (truncated)'}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
};
