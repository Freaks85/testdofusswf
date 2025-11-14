import React from 'react';
import { ResourceType, ResourceInfo } from '../types';
import { invoke } from '@tauri-apps/api/core';

// Efficient base64 encoding for binary data
function arrayBufferToBase64(buffer: Uint8Array): string {
  let binary = '';
  const len = buffer.byteLength;
  for (let i = 0; i < len; i++) {
    binary += String.fromCharCode(buffer[i]);
  }
  return btoa(binary);
}

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
  const [isEditing, setIsEditing] = React.useState(false);
  const [editedCode, setEditedCode] = React.useState<string>('');
  const [editedText, setEditedText] = React.useState<string>('');
  const [saving, setSaving] = React.useState(false);

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
          <div className="panel-content" style={{ padding: 0 }}>
            <div style={{
              background: 'var(--bg-secondary)',
              borderBottom: '1px solid var(--border-default)',
              padding: '12px 16px',
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center'
            }}>
              <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                <span style={{ fontSize: '20px' }}>📜</span>
                <span style={{ color: 'var(--text-accent)', fontWeight: 600 }}>
                  {resourceInfo.metadata?.script_type || 'Script'} Code
                </span>
                <span style={{
                  fontSize: '11px',
                  padding: '2px 8px',
                  background: 'var(--bg-tertiary)',
                  borderRadius: '4px',
                  color: 'var(--text-muted)'
                }}>
                  {decompiled.split('\n').length} lines
                </span>
              </div>
              <button
                onClick={() => {
                  setIsEditing(!isEditing);
                  setEditedCode(decompiled);
                }}
                className="btn-secondary"
                style={{ fontSize: '13px', padding: '6px 12px' }}
              >
                {isEditing ? '👁️ View' : '✏️ Edit'}
              </button>
            </div>
            {isEditing ? (
              <textarea
                value={editedCode}
                onChange={(e) => setEditedCode(e.target.value)}
                style={{
                  width: '100%',
                  height: '600px',
                  background: 'var(--bg-primary)',
                  color: 'var(--text-primary)',
                  border: 'none',
                  padding: '16px',
                  fontFamily: 'Consolas, Monaco, "Courier New", monospace',
                  fontSize: '13px',
                  lineHeight: '1.6',
                  resize: 'none',
                  outline: 'none'
                }}
              />
            ) : (
              <pre className="code-editor" style={{
                margin: 0,
                padding: '16px',
                whiteSpace: 'pre-wrap',
                fontFamily: 'Consolas, Monaco, "Courier New", monospace',
                fontSize: '13px',
                lineHeight: '1.6',
                maxHeight: '600px',
                overflow: 'auto'
              }}>
                {decompiled}
              </pre>
            )}
          </div>
        )}

        {resourceType === 'images' && data && (
          <div className="panel-content" style={{ textAlign: 'center', padding: '20px' }}>
            <div style={{
              display: 'inline-block',
              maxWidth: '100%',
              background: 'repeating-conic-gradient(#808080 0% 25%, transparent 0% 50%) 50% / 20px 20px',
              padding: '10px',
              borderRadius: '4px',
              boxShadow: '0 2px 8px rgba(0,0,0,0.3)'
            }}>
              <img
                src={`data:image/png;base64,${arrayBufferToBase64(data)}`}
                alt={`Image ${resourceId}`}
                style={{
                  maxWidth: '100%',
                  maxHeight: '600px',
                  display: 'block',
                  imageRendering: 'crisp-edges'
                }}
                onError={() => {
                  setError('Failed to load image - data may be corrupted or in unsupported format');
                }}
              />
            </div>
            <div style={{ marginTop: '16px', color: 'var(--text-muted)', fontSize: '12px' }}>
              <p>Dimensions: {resourceInfo.metadata?.width || '?'} x {resourceInfo.metadata?.height || '?'} • Format: {resourceInfo.metadata?.format || 'Unknown'}</p>
            </div>
          </div>
        )}

        {resourceType === 'sounds' && data && (
          <div className="panel-content" style={{ padding: '20px' }}>
            <div style={{
              background: 'var(--bg-tertiary)',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid var(--border-default)'
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' }}>
                <span style={{ fontSize: '32px' }}>🔊</span>
                <div>
                  <h3 style={{ margin: 0, color: 'var(--text-accent)' }}>Sound Resource</h3>
                  <p style={{ margin: '4px 0 0 0', color: 'var(--text-muted)', fontSize: '13px' }}>
                    Format: {resourceInfo.metadata?.format || 'Unknown'} • Sample Rate: {resourceInfo.metadata?.sample_rate || '?'} Hz
                  </p>
                </div>
              </div>

              <div style={{
                background: 'var(--bg-primary)',
                padding: '16px',
                borderRadius: '6px',
                marginTop: '16px'
              }}>
                <h4 style={{ margin: '0 0 12px 0', color: 'var(--text-secondary)' }}>Audio Properties</h4>
                <div style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '8px 16px', fontSize: '13px' }}>
                  <span style={{ color: 'var(--text-muted)' }}>Channels:</span>
                  <span style={{ color: 'var(--text-secondary)' }}>{resourceInfo.metadata?.stereo === 'Yes' ? 'Stereo (2)' : 'Mono (1)'}</span>

                  <span style={{ color: 'var(--text-muted)' }}>Data Size:</span>
                  <span style={{ color: 'var(--text-secondary)' }}>{(data.length / 1024).toFixed(2)} KB</span>

                  <span style={{ color: 'var(--text-muted)' }}>Format:</span>
                  <span style={{ color: 'var(--text-secondary)' }}>{resourceInfo.metadata?.format || 'Unknown'}</span>
                </div>
              </div>

              <div style={{
                marginTop: '16px',
                padding: '12px',
                background: 'rgba(255, 193, 7, 0.1)',
                border: '1px solid rgba(255, 193, 7, 0.3)',
                borderRadius: '6px',
                color: 'var(--text-secondary)',
                fontSize: '13px'
              }}>
                <strong>💡 Tip:</strong> Audio playback coming soon! You can export this sound to play it externally.
              </div>
            </div>
          </div>
        )}

        {resourceType === 'sprites' && (
          <div className="panel-content" style={{ padding: '20px' }}>
            <div style={{
              background: 'var(--bg-tertiary)',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid var(--border-default)'
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' }}>
                <span style={{ fontSize: '32px' }}>🎬</span>
                <div>
                  <h3 style={{ margin: 0, color: 'var(--text-accent)' }}>Sprite/MovieClip</h3>
                  <p style={{ margin: '4px 0 0 0', color: 'var(--text-muted)', fontSize: '13px' }}>
                    Frames: {resourceInfo.metadata?.frame_count || '?'} • ID: {resourceId}
                  </p>
                </div>
              </div>

              <div style={{
                background: 'var(--bg-primary)',
                padding: '16px',
                borderRadius: '6px',
                marginTop: '16px'
              }}>
                <h4 style={{ margin: '0 0 12px 0', color: 'var(--text-secondary)' }}>Structure</h4>
                <ul style={{ margin: 0, paddingLeft: '20px', color: 'var(--text-secondary)' }}>
                  <li>Contains timeline with {resourceInfo.metadata?.frame_count || 0} frame(s)</li>
                  <li>Nested tags and display list objects</li>
                  <li>Used for animations and complex graphics</li>
                </ul>
              </div>

              <div style={{
                marginTop: '16px',
                padding: '12px',
                background: 'rgba(255, 193, 7, 0.1)',
                border: '1px solid rgba(255, 193, 7, 0.3)',
                borderRadius: '6px',
                color: 'var(--text-secondary)',
                fontSize: '13px'
              }}>
                <strong>💡 Tip:</strong> Sprite preview with frame visualization coming soon!
              </div>
            </div>
          </div>
        )}

        {resourceType === 'texts' && (
          <div className="panel-content" style={{ padding: '20px' }}>
            <div style={{
              background: 'var(--bg-tertiary)',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid var(--border-default)'
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px', justifyContent: 'space-between' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                  <span style={{ fontSize: '32px' }}>📝</span>
                  <div>
                    <h3 style={{ margin: 0, color: 'var(--text-accent)' }}>Text Field</h3>
                    <p style={{ margin: '4px 0 0 0', color: 'var(--text-muted)', fontSize: '13px' }}>
                      ID: {resourceId}
                    </p>
                  </div>
                </div>
                {!isEditing && (
                  <button
                    onClick={() => {
                      setIsEditing(true);
                      setEditedText(resourceInfo.name || '');
                    }}
                    className="btn-primary"
                    style={{ fontSize: '13px', padding: '6px 12px' }}
                  >
                    ✏️ Edit Text
                  </button>
                )}
              </div>

              <div style={{
                background: 'var(--bg-primary)',
                padding: '16px',
                borderRadius: '6px',
                marginTop: '16px'
              }}>
                <h4 style={{ margin: '0 0 12px 0', color: 'var(--text-secondary)' }}>Text Content</h4>
                {isEditing ? (
                  <>
                    <textarea
                      value={editedText}
                      onChange={(e) => setEditedText(e.target.value)}
                      style={{
                        width: '100%',
                        minHeight: '200px',
                        padding: '12px',
                        background: 'var(--bg-secondary)',
                        border: '1px solid var(--border-default)',
                        borderRadius: '4px',
                        fontSize: '14px',
                        color: 'var(--text-primary)',
                        fontFamily: 'inherit',
                        resize: 'vertical'
                      }}
                      placeholder="Enter text content..."
                    />
                    <div style={{ display: 'flex', gap: '8px', marginTop: '12px' }}>
                      <button
                        onClick={async () => {
                          setSaving(true);
                          try {
                            await invoke('update_text', {
                              textId: resourceId,
                              newText: editedText
                            });
                            alert('Text updated successfully! Remember to save the SWF file.');
                            setIsEditing(false);
                            // Update display
                            resourceInfo.name = editedText;
                          } catch (err) {
                            alert(`Failed to update text: ${err}`);
                          } finally {
                            setSaving(false);
                          }
                        }}
                        className="btn-primary"
                        disabled={saving}
                      >
                        {saving ? '💾 Saving...' : '💾 Save Changes'}
                      </button>
                      <button
                        onClick={() => setIsEditing(false)}
                        className="btn-secondary"
                        disabled={saving}
                      >
                        ❌ Cancel
                      </button>
                    </div>
                  </>
                ) : (
                  <div style={{
                    padding: '12px',
                    background: 'var(--bg-secondary)',
                    borderRadius: '4px',
                    fontSize: '14px',
                    color: 'var(--text-primary)',
                    wordWrap: 'break-word'
                  }}>
                    {resourceInfo.name || 'No text content'}
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {resourceType === 'fonts' && (
          <div className="panel-content" style={{ padding: '20px' }}>
            <div style={{
              background: 'var(--bg-tertiary)',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid var(--border-default)'
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' }}>
                <span style={{ fontSize: '32px' }}>🔤</span>
                <div>
                  <h3 style={{ margin: 0, color: 'var(--text-accent)' }}>Font Resource</h3>
                  <p style={{ margin: '4px 0 0 0', color: 'var(--text-muted)', fontSize: '13px' }}>
                    {resourceInfo.name || `Font_${resourceId}`}
                  </p>
                </div>
              </div>

              <div style={{
                background: 'var(--bg-primary)',
                padding: '16px',
                borderRadius: '6px',
                marginTop: '16px'
              }}>
                <h4 style={{ margin: '0 0 12px 0', color: 'var(--text-secondary)' }}>Font Properties</h4>
                <div style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '8px 16px', fontSize: '13px' }}>
                  <span style={{ color: 'var(--text-muted)' }}>Glyphs:</span>
                  <span style={{ color: 'var(--text-secondary)' }}>{resourceInfo.metadata?.num_glyphs || 0}</span>

                  <span style={{ color: 'var(--text-muted)' }}>Data Size:</span>
                  <span style={{ color: 'var(--text-secondary)' }}>{(resourceInfo.size / 1024).toFixed(2)} KB</span>
                </div>
              </div>
            </div>
          </div>
        )}

        {resourceType === 'shapes' && (
          <div className="panel-content" style={{ padding: '20px' }}>
            <div style={{
              background: 'var(--bg-tertiary)',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid var(--border-default)'
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' }}>
                <span style={{ fontSize: '32px' }}>⬢</span>
                <div>
                  <h3 style={{ margin: 0, color: 'var(--text-accent)' }}>Vector Shape</h3>
                  <p style={{ margin: '4px 0 0 0', color: 'var(--text-muted)', fontSize: '13px' }}>
                    ID: {resourceId}
                  </p>
                </div>
              </div>

              <div style={{
                background: 'var(--bg-primary)',
                padding: '16px',
                borderRadius: '6px',
                marginTop: '16px'
              }}>
                <h4 style={{ margin: '0 0 12px 0', color: 'var(--text-secondary)' }}>Shape Info</h4>
                <ul style={{ margin: 0, paddingLeft: '20px', color: 'var(--text-secondary)' }}>
                  <li>Vector graphics definition</li>
                  <li>Contains paths, fills, and strokes</li>
                  <li>Size: {(resourceInfo.size / 1024).toFixed(2)} KB</li>
                </ul>
              </div>

              <div style={{
                marginTop: '16px',
                padding: '12px',
                background: 'rgba(255, 193, 7, 0.1)',
                border: '1px solid rgba(255, 193, 7, 0.3)',
                borderRadius: '6px',
                color: 'var(--text-secondary)',
                fontSize: '13px'
              }}>
                <strong>💡 Tip:</strong> SVG rendering preview coming soon!
              </div>
            </div>
          </div>
        )}

        {resourceType === 'binary_data' && data && (
          <div className="panel-content" style={{ padding: '20px' }}>
            <div style={{
              background: 'var(--bg-tertiary)',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid var(--border-default)'
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' }}>
                <span style={{ fontSize: '32px' }}>📦</span>
                <div>
                  <h3 style={{ margin: 0, color: 'var(--text-accent)' }}>Binary Data</h3>
                  <p style={{ margin: '4px 0 0 0', color: 'var(--text-muted)', fontSize: '13px' }}>
                    ID: {resourceId} • Size: {(data.length / 1024).toFixed(2)} KB
                  </p>
                </div>
              </div>

              <div style={{
                background: 'var(--bg-primary)',
                padding: '16px',
                borderRadius: '6px',
                marginTop: '16px'
              }}>
                <h4 style={{ margin: '0 0 12px 0', color: 'var(--text-secondary)' }}>Hex Dump (first 256 bytes)</h4>
                <pre className="code-editor" style={{
                  fontSize: '11px',
                  margin: 0,
                  fontFamily: 'Consolas, Monaco, monospace',
                  lineHeight: '1.5'
                }}>
                  {Array.from(data.slice(0, 256)).map((b, i) =>
                    (i % 16 === 0 ? '\n' : '') + b.toString(16).padStart(2, '0') + ' '
                  )}
                  {data.length > 256 && '\n... (truncated)'}
                </pre>
              </div>
            </div>
          </div>
        )}

        {!resourceType.match(/scripts|images|sounds|sprites|texts|fonts|shapes|binary_data/) && data && (
          <div className="panel-content">
            <p className="text-secondary">Unknown resource type ({data.length} bytes)</p>
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
