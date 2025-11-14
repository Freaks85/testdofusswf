import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { SWFFile, ResourceInfo, ResourceType } from './types';
import { FileTree } from './components/FileTree';
import { Viewer } from './components/Viewer';
import { SearchPanel } from './components/SearchPanel';
import './App.css';

// Component to show debug info about tags
const TagsDebugView: React.FC = () => {
  const [tags, setTags] = React.useState<string[]>([]);
  const [showTags, setShowTags] = React.useState(false);
  const [loading, setLoading] = React.useState(false);

  const loadTags = async () => {
    setLoading(true);
    try {
      const tagsData = await invoke<string[]>('get_tags_debug', {});
      setTags(tagsData);
      setShowTags(true);
    } catch (err) {
      console.error('Failed to load tags:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ marginTop: '20px' }}>
      <button
        onClick={showTags ? () => setShowTags(false) : loadTags}
        disabled={loading}
        className="btn-warning"
      >
        {loading ? 'Loading...' : showTags ? '🔽 Hide Tags' : '🔍 Show All Tags (Debug)'}
      </button>

      {showTags && tags.length > 0 && (
        <div className="panel fade-in" style={{ marginTop: '12px', maxHeight: '400px' }}>
          <div className="panel-header">
            <span>Detected Tags ({tags.length})</span>
          </div>
          <div className="panel-content overflow-auto" style={{ maxHeight: '350px' }}>
            <div className="monospace" style={{ fontSize: '13px' }}>
              {tags.map((tag, i) => (
                <div key={i} style={{
                  padding: '6px 0',
                  borderBottom: i < tags.length - 1 ? '1px solid var(--border-default)' : 'none'
                }}>
                  {tag}
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

interface AppState {
  swfFile: SWFFile | null;
  resources: {
    images: ResourceInfo[];
    sounds: ResourceInfo[];
    sprites: ResourceInfo[];
    scripts: ResourceInfo[];
    texts: ResourceInfo[];
    fonts: ResourceInfo[];
    shapes: ResourceInfo[];
    binary_data: ResourceInfo[];
  };
  selectedResource: { type: ResourceType; id: number; info: ResourceInfo } | null;
  loading: boolean;
  error: string | null;
  swfInfo: Record<string, string> | null;
}

function App() {
  const [state, setState] = React.useState<AppState>({
    swfFile: null,
    resources: {
      images: [],
      sounds: [],
      sprites: [],
      scripts: [],
      texts: [],
      fonts: [],
      shapes: [],
      binary_data: [],
    },
    selectedResource: null,
    loading: false,
    error: null,
    swfInfo: null,
  });

  const [isDragging, setIsDragging] = React.useState(false);
  const [searchVisible, setSearchVisible] = React.useState(false);

  // Handle Ctrl+F keyboard shortcut
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
        e.preventDefault();
        if (state.swfFile) {
          setSearchVisible(true);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [state.swfFile]);

  const handleSearchResultSelect = (type: string, id: number) => {
    const resourceType = type as ResourceType;
    handleSelectResource(resourceType, id);
  };

  const loadSWFFile = async (path: string) => {
    setState((prev) => ({ ...prev, loading: true, error: null }));

    try {
      // Open the SWF file
      const swf = await invoke<SWFFile>('open_swf', { path });

      // Get resources - all types
      const images = await invoke<ResourceInfo[]>('get_resources', { resourceType: 'images' });
      const sounds = await invoke<ResourceInfo[]>('get_resources', { resourceType: 'sounds' });
      const sprites = await invoke<ResourceInfo[]>('get_resources', { resourceType: 'sprites' });
      const scripts = await invoke<ResourceInfo[]>('get_resources', { resourceType: 'scripts' });
      const texts = await invoke<ResourceInfo[]>('get_resources', { resourceType: 'texts' });
      const fonts = await invoke<ResourceInfo[]>('get_resources', { resourceType: 'fonts' });
      const shapes = await invoke<ResourceInfo[]>('get_resources', { resourceType: 'shapes' });
      const binary_data = await invoke<ResourceInfo[]>('get_resources', { resourceType: 'binary_data' });

      // Get SWF info
      const info = await invoke<Record<string, string>>('get_swf_info', {});

      // Debug logging
      console.log('=== SWF LOADED ===');
      console.log('File:', path);
      console.log('Tags:', swf.tags.length);
      console.log('Images:', images.length);
      console.log('Sounds:', sounds.length);
      console.log('Sprites:', sprites.length);
      console.log('Scripts:', scripts.length);
      console.log('Texts:', texts.length);
      console.log('Fonts:', fonts.length);
      console.log('Shapes:', shapes.length);
      console.log('Binary Data:', binary_data.length);
      console.log('Info:', info);
      console.log('==================');

      setState({
        swfFile: swf,
        resources: { images, sounds, sprites, scripts, texts, fonts, shapes, binary_data },
        selectedResource: null,
        loading: false,
        error: null,
        swfInfo: info,
      });
    } catch (err) {
      setState((prev) => ({
        ...prev,
        loading: false,
        error: String(err),
      }));
    }
  };

  const handleOpenFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'SWF Files',
            extensions: ['swf'],
          },
        ],
      });

      if (selected) {
        await loadSWFFile(selected as string);
      }
    } catch (err) {
      setState((prev) => ({ ...prev, error: String(err) }));
    }
  };

  const handleExportAll = async () => {
    try {
      const { open: openDir } = await import('@tauri-apps/plugin-dialog');
      const dir = await openDir({
        directory: true,
      });

      if (dir) {
        const count = await invoke<number>('export_all_resources', { outputDir: dir });
        alert(`Exported ${count} resources successfully!`);
      }
    } catch (err) {
      alert(`Export failed: ${err}`);
    }
  };

  const handleSaveSWF = async () => {
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const filePath = await save({
        defaultPath: state.swfFile?.path || 'modified.swf',
        filters: [
          {
            name: 'SWF Files',
            extensions: ['swf'],
          },
        ],
      });

      if (filePath) {
        await invoke('save_swf_as', { outputPath: filePath });
        alert('SWF file saved successfully!');
      }
    } catch (err) {
      alert(`Save failed: ${err}`);
    }
  };

  const handleSelectResource = (type: ResourceType, id: number) => {
    const resourceList = state.resources[type as keyof typeof state.resources];
    if (!resourceList) return;

    const info = resourceList.find((r: ResourceInfo) => r.id === id);

    if (info) {
      setState((prev) => ({
        ...prev,
        selectedResource: { type, id, info },
      }));
    }
  };

  // Drag and drop handlers
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    const files = Array.from(e.dataTransfer.files);
    const swfFile = files.find((f) => f.name.endsWith('.swf'));

    if (swfFile) {
      // @ts-ignore - path is available in Tauri
      const path = swfFile.path;
      if (path) {
        await loadSWFFile(path);
      }
    } else {
      setState((prev) => ({ ...prev, error: 'Please drop a .swf file' }));
    }
  };

  return (
    <div
      className="app-container flex flex-col h-full"
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* Header */}
      <div className="app-header flex justify-between items-center p-3" style={{
        background: 'var(--button-primary)',
        borderBottom: '1px solid var(--border-default)',
      }}>
        <h2 style={{ margin: 0 }}>SWF Editor</h2>
        <div className="flex gap-2">
          <button onClick={handleOpenFile} className="btn-secondary">
            📁 Open SWF
          </button>
          {state.swfFile && (
            <>
              <button onClick={handleExportAll} className="btn-success">
                📤 Export All
              </button>
              <button onClick={handleSaveSWF} className="btn-primary">
                💾 Save SWF
              </button>
            </>
          )}
        </div>
      </div>

      {/* Main content */}
      <div className="flex h-full overflow-hidden" style={{ flex: 1 }}>
        {state.loading && (
          <div className="flex items-center justify-center w-full">
            <div className="flex items-center gap-2">
              <div className="spinner"></div>
              <span>Loading SWF file...</span>
            </div>
          </div>
        )}

        {state.error && (
          <div className="flex items-center justify-center w-full">
            <div className="panel" style={{ maxWidth: '600px' }}>
              <div className="panel-header text-error">
                <h3 style={{ margin: 0 }}>Error</h3>
              </div>
              <div className="panel-content">
                <p className="text-error">{state.error}</p>
              </div>
            </div>
          </div>
        )}

        {!state.loading && !state.error && !state.swfFile && (
          <div
            className="flex items-center justify-center w-full"
            style={{
              background: isDragging ? 'var(--selection-bg)' : 'transparent',
              border: isDragging ? '2px dashed var(--border-focus)' : 'none',
              transition: 'all 0.2s',
            }}
          >
            <div style={{ textAlign: 'center', padding: '40px' }}>
              <h2>Drop a SWF file here</h2>
              <p className="text-secondary">or click "Open SWF" button above</p>
            </div>
          </div>
        )}

        {!state.loading && !state.error && state.swfFile && (
          <>
            {/* File Tree */}
            <div style={{ width: '300px', borderRight: '1px solid var(--border-default)' }} className="overflow-auto bg-secondary">
              <FileTree
                resources={state.resources}
                onSelectResource={handleSelectResource}
                selectedResource={
                  state.selectedResource
                    ? { type: state.selectedResource.type, id: state.selectedResource.id }
                    : undefined
                }
              />
            </div>

            {/* Viewer */}
            <div className="overflow-auto bg-primary" style={{ flex: 1 }}>
              {state.selectedResource ? (
                <Viewer
                  resourceType={state.selectedResource.type}
                  resourceId={state.selectedResource.id}
                  resourceInfo={state.selectedResource.info}
                />
              ) : (
                <div className="p-4">
                  <h2>SWF File Information</h2>
                  {state.swfInfo && (
                    <div className="panel" style={{ marginBottom: '20px' }}>
                      <div className="panel-content">
                        {Object.entries(state.swfInfo).map(([key, value]) => (
                          <div key={key} style={{ marginBottom: '8px' }}>
                            <strong className="text-accent">{key.replace(/_/g, ' ')}:</strong>{' '}
                            <span className="text-secondary">{value}</span>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  <TagsDebugView />

                  <p className="text-muted" style={{ marginTop: '20px' }}>
                    Select a resource from the tree to view it.
                  </p>
                </div>
              )}
            </div>
          </>
        )}
      </div>

      {/* Status bar */}
      {state.swfFile && (
        <div
          className="p-2 text-muted"
          style={{
            background: 'var(--bg-tertiary)',
            borderTop: '1px solid var(--border-default)',
            fontSize: '12px',
          }}
        >
          <span className="status-dot status-success"></span>
          <strong>Status:</strong> Ready | <strong>File:</strong> {state.swfFile.path} | <strong>Tags:</strong> {state.swfFile.tags.length} |
          <strong> Images:</strong> {state.resources.images.length} |
          <strong>Sounds:</strong> {state.resources.sounds.length} |
          <strong>Sprites:</strong> {state.resources.sprites.length} |
          <strong> Scripts:</strong> {state.resources.scripts.length} |
          <strong>Texts:</strong> {state.resources.texts.length} |
          <strong>Fonts:</strong> {state.resources.fonts.length} |
          <strong>Shapes:</strong> {state.resources.shapes.length} |
          <strong>Binary:</strong> {state.resources.binary_data.length}
        </div>
      )}

      {/* Search Panel */}
      <SearchPanel
        visible={searchVisible}
        onClose={() => setSearchVisible(false)}
        onSelectResult={handleSearchResultSelect}
      />
    </div>
  );
}

export default App;
