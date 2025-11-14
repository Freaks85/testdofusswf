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
    return <div style={{ padding: '20px' }}>Loading...</div>;
  }

  if (error) {
    return <div style={{ padding: '20px', color: 'red' }}>Error: {error}</div>;
  }

  return (
    <div style={{ padding: '20px', height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={{ marginBottom: '16px' }}>
        <h2>{resourceInfo.name || `${resourceType}_${resourceId}`}</h2>
        <div style={{ background: '#f5f5f5', padding: '12px', borderRadius: '4px' }}>
          <div><strong>ID:</strong> {resourceId}</div>
          <div><strong>Type:</strong> {resourceInfo.resource_type}</div>
          <div><strong>Size:</strong> {(resourceInfo.size / 1024).toFixed(2)} KB</div>
          {resourceInfo.metadata && Object.entries(resourceInfo.metadata).map(([key, value]) => (
            <div key={key}><strong>{key}:</strong> {value}</div>
          ))}
        </div>
        <button
          onClick={handleExport}
          style={{
            marginTop: '12px',
            padding: '8px 16px',
            background: '#2196F3',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Export
        </button>
      </div>

      <div style={{ flex: 1, overflow: 'auto', background: '#fafafa', padding: '12px', borderRadius: '4px' }}>
        {resourceType === 'scripts' && decompiled && (
          <pre style={{ margin: 0, fontSize: '12px', fontFamily: 'monospace', whiteSpace: 'pre-wrap' }}>
            {decompiled}
          </pre>
        )}

        {resourceType === 'images' && data && (
          <div style={{ textAlign: 'center' }}>
            <img
              src={`data:image/png;base64,${btoa(String.fromCharCode(...data))}`}
              alt={`Image ${resourceId}`}
              style={{ maxWidth: '100%', maxHeight: '600px' }}
              onError={() => {
                setError('Failed to load image');
              }}
            />
          </div>
        )}

        {resourceType === 'sounds' && data && (
          <div>
            <p>Sound resource (playback not yet implemented)</p>
            <p>Raw data size: {data.length} bytes</p>
          </div>
        )}

        {resourceType === 'sprites' && (
          <div>
            <p>Sprite preview not yet implemented</p>
          </div>
        )}

        {!resourceType.match(/scripts|images|sounds|sprites/) && data && (
          <div>
            <p>Binary data ({data.length} bytes)</p>
            <pre style={{ fontSize: '10px', fontFamily: 'monospace' }}>
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
