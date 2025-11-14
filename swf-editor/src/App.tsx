import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { SWFFile, ResourceInfo, ResourceType } from './types';
import { FileTree } from './components/FileTree';
import { Viewer } from './components/Viewer';
import './App.css';

interface AppState {
  swfFile: SWFFile | null;
  resources: {
    images: ResourceInfo[];
    sounds: ResourceInfo[];
    sprites: ResourceInfo[];
    scripts: ResourceInfo[];
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
    },
    selectedResource: null,
    loading: false,
    error: null,
    swfInfo: null,
  });

  const [isDragging, setIsDragging] = React.useState(false);

  const loadSWFFile = async (path: string) => {
    setState((prev) => ({ ...prev, loading: true, error: null }));

    try {
      // Open the SWF file
      const swf = await invoke<SWFFile>('open_swf', { path });

      // Get resources
      const images = await invoke<ResourceInfo[]>('get_resources', { resourceType: 'images' });
      const sounds = await invoke<ResourceInfo[]>('get_resources', { resourceType: 'sounds' });
      const sprites = await invoke<ResourceInfo[]>('get_resources', { resourceType: 'sprites' });
      const scripts = await invoke<ResourceInfo[]>('get_resources', { resourceType: 'scripts' });

      // Get SWF info
      const info = await invoke<Record<string, string>>('get_swf_info', {});

      setState({
        swfFile: swf,
        resources: { images, sounds, sprites, scripts },
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

  const handleSelectResource = (type: ResourceType, id: number) => {
    const resourceList = state.resources[type];
    const info = resourceList.find((r) => r.id === id);

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
      style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* Header */}
      <div
        style={{
          padding: '12px 16px',
          background: '#2196F3',
          color: 'white',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <h1 style={{ margin: 0, fontSize: '20px' }}>SWF Editor</h1>
        <div>
          <button
            onClick={handleOpenFile}
            style={{
              padding: '8px 16px',
              marginRight: '8px',
              background: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            📁 Open SWF
          </button>
          {state.swfFile && (
            <button
              onClick={handleExportAll}
              style={{
                padding: '8px 16px',
                background: '#4CAF50',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              📤 Export All
            </button>
          )}
        </div>
      </div>

      {/* Main content */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {state.loading && (
          <div
            style={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <div>Loading SWF file...</div>
          </div>
        )}

        {state.error && (
          <div
            style={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: 'red',
            }}
          >
            <div>
              <h2>Error</h2>
              <p>{state.error}</p>
            </div>
          </div>
        )}

        {!state.loading && !state.error && !state.swfFile && (
          <div
            style={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              background: isDragging ? '#e3f2fd' : 'transparent',
              border: isDragging ? '2px dashed #2196F3' : 'none',
              transition: 'all 0.2s',
            }}
          >
            <div style={{ textAlign: 'center', padding: '40px' }}>
              <h2>Drop a SWF file here</h2>
              <p>or click "Open SWF" button above</p>
            </div>
          </div>
        )}

        {!state.loading && !state.error && state.swfFile && (
          <>
            {/* File Tree */}
            <div style={{ width: '300px', overflow: 'auto' }}>
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
            <div style={{ flex: 1, overflow: 'auto' }}>
              {state.selectedResource ? (
                <Viewer
                  resourceType={state.selectedResource.type}
                  resourceId={state.selectedResource.id}
                  resourceInfo={state.selectedResource.info}
                />
              ) : (
                <div style={{ padding: '20px' }}>
                  <h2>SWF File Information</h2>
                  {state.swfInfo && (
                    <div style={{ background: '#f5f5f5', padding: '16px', borderRadius: '4px' }}>
                      {Object.entries(state.swfInfo).map(([key, value]) => (
                        <div key={key} style={{ marginBottom: '8px' }}>
                          <strong>{key.replace(/_/g, ' ')}:</strong> {value}
                        </div>
                      ))}
                    </div>
                  )}
                  <p style={{ marginTop: '20px', color: '#666' }}>
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
          style={{
            padding: '8px 16px',
            background: '#f5f5f5',
            borderTop: '1px solid #ddd',
            fontSize: '12px',
          }}
        >
          Status: Ready | File: {state.swfFile.path} | Tags: {state.swfFile.tags.length} | Images:{' '}
          {state.resources.images.length} | Sounds: {state.resources.sounds.length} | Scripts:{' '}
          {state.resources.scripts.length}
        </div>
      )}
    </div>
  );
}

export default App;
