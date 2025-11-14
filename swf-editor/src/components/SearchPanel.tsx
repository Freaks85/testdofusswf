import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface SearchResult {
  resource_type: string;
  resource_id: number;
  resource_name: string;
  match_type: string; // "name", "content", "id"
  preview: string;
  line_number?: number;
}

interface SearchPanelProps {
  visible: boolean;
  onClose: () => void;
  onSelectResult: (type: string, id: number) => void;
}

export const SearchPanel: React.FC<SearchPanelProps> = ({ visible, onClose, onSelectResult }) => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [searching, setSearching] = useState(false);
  const [searchMode, setSearchMode] = useState<'all' | 'name' | 'content'>('all');
  const [caseSensitive, setCaseSensitive] = useState(false);

  const handleSearch = async () => {
    if (!query.trim()) {
      setResults([]);
      return;
    }

    setSearching(true);
    try {
      const searchResults = await invoke<SearchResult[]>('search_swf', {
        query: query,
        searchMode: searchMode,
        caseSensitive: caseSensitive,
      });
      setResults(searchResults);
    } catch (error) {
      console.error('Search failed:', error);
      setResults([]);
    } finally {
      setSearching(false);
    }
  };

  // Search when Enter is pressed
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch();
    } else if (e.key === 'Escape') {
      onClose();
    }
  };

  useEffect(() => {
    if (visible) {
      // Focus search input when panel opens
      const input = document.getElementById('search-input');
      if (input) {
        setTimeout(() => input.focus(), 100);
      }
    }
  }, [visible]);

  if (!visible) {
    return null;
  }

  return (
    <div className="search-panel-overlay" onClick={onClose}>
      <div className="search-panel panel" onClick={(e) => e.stopPropagation()}>
        <div className="panel-header flex justify-between items-center">
          <h3 style={{ margin: 0 }}>Search SWF</h3>
          <button onClick={onClose} className="btn-icon">✕</button>
        </div>

        <div className="panel-content">
          {/* Search Input */}
          <div className="flex gap-2" style={{ marginBottom: '12px' }}>
            <input
              id="search-input"
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Search for resources, text, code..."
              className="flex-1"
              style={{
                padding: '8px 12px',
                background: 'var(--bg-input)',
                border: '1px solid var(--border-default)',
                borderRadius: '4px',
                color: 'var(--text-primary)',
              }}
            />
            <button
              onClick={handleSearch}
              disabled={searching || !query.trim()}
              className="btn-primary"
            >
              {searching ? '🔍 Searching...' : '🔍 Search'}
            </button>
          </div>

          {/* Search Options */}
          <div className="flex gap-4" style={{ marginBottom: '16px' }}>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                checked={searchMode === 'all'}
                onChange={() => setSearchMode('all')}
              />
              <span className="text-secondary">All</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                checked={searchMode === 'name'}
                onChange={() => setSearchMode('name')}
              />
              <span className="text-secondary">Names Only</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                checked={searchMode === 'content'}
                onChange={() => setSearchMode('content')}
              />
              <span className="text-secondary">Content Only</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={caseSensitive}
                onChange={(e) => setCaseSensitive(e.target.checked)}
              />
              <span className="text-secondary">Case Sensitive</span>
            </label>
          </div>

          {/* Results */}
          <div className="search-results" style={{ maxHeight: '400px', overflow: 'auto' }}>
            {results.length === 0 && query && !searching && (
              <div className="text-muted text-center" style={{ padding: '40px' }}>
                No results found for "{query}"
              </div>
            )}

            {results.length > 0 && (
              <>
                <div className="text-secondary" style={{ marginBottom: '8px' }}>
                  Found {results.length} result{results.length !== 1 ? 's' : ''}
                </div>
                {results.map((result, i) => (
                  <div
                    key={i}
                    className="search-result-item cursor-pointer"
                    onClick={() => {
                      onSelectResult(result.resource_type, result.resource_id);
                      onClose();
                    }}
                    style={{
                      padding: '10px 12px',
                      marginBottom: '4px',
                      background: 'var(--bg-tertiary)',
                      borderRadius: '4px',
                      borderLeft: `3px solid ${getResourceColor(result.resource_type)}`,
                      transition: 'all 0.2s',
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.background = 'var(--bg-elevated)';
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.background = 'var(--bg-tertiary)';
                    }}
                  >
                    <div className="flex justify-between items-start">
                      <div style={{ flex: 1 }}>
                        <div className="flex items-center gap-2">
                          <span className={`badge badge-${result.resource_type}`}>
                            {result.resource_type}
                          </span>
                          <span className="text-primary font-medium">
                            {result.resource_name || `${result.resource_type}_${result.resource_id}`}
                          </span>
                        </div>
                        <div className="text-muted" style={{ fontSize: '11px', marginTop: '4px' }}>
                          Match in: {result.match_type}
                          {result.line_number && ` (line ${result.line_number})`}
                        </div>
                        {result.preview && (
                          <div
                            className="monospace text-secondary"
                            style={{
                              fontSize: '11px',
                              marginTop: '6px',
                              padding: '4px 8px',
                              background: 'var(--bg-primary)',
                              borderRadius: '2px',
                              whiteSpace: 'pre-wrap',
                              wordBreak: 'break-all',
                            }}
                          >
                            {result.preview.length > 100
                              ? result.preview.substring(0, 100) + '...'
                              : result.preview}
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </>
            )}
          </div>
        </div>

        <div className="panel-footer text-muted" style={{ fontSize: '11px' }}>
          Press Enter to search, Esc to close
        </div>
      </div>
    </div>
  );
};

function getResourceColor(type: string): string {
  switch (type) {
    case 'images':
      return '#f06292';
    case 'sounds':
      return '#ba68c8';
    case 'sprites':
      return '#4fc3f7';
    case 'scripts':
      return '#aed581';
    case 'fonts':
      return '#ffd54f';
    case 'texts':
      return '#ff8a65';
    case 'shapes':
      return '#64b5f6';
    default:
      return '#999';
  }
}
