import React from 'react';
import { ResourceInfo, ResourceType } from '../types';

interface FileTreeProps {
  resources: {
    images: ResourceInfo[];
    sounds: ResourceInfo[];
    sprites: ResourceInfo[];
    scripts: ResourceInfo[];
  };
  onSelectResource: (type: ResourceType, id: number) => void;
  selectedResource?: { type: ResourceType; id: number };
}

export const FileTree: React.FC<FileTreeProps> = ({
  resources,
  onSelectResource,
  selectedResource,
}) => {
  const [expandedSections, setExpandedSections] = React.useState<Set<ResourceType>>(
    new Set(['images', 'sounds', 'sprites', 'scripts'])
  );

  const toggleSection = (section: ResourceType) => {
    const newExpanded = new Set(expandedSections);
    if (newExpanded.has(section)) {
      newExpanded.delete(section);
    } else {
      newExpanded.add(section);
    }
    setExpandedSections(newExpanded);
  };

  const renderResourceList = (type: ResourceType, items: ResourceInfo[], icon: string) => {
    const isExpanded = expandedSections.has(type);

    return (
      <div key={type} style={{ marginBottom: '8px' }}>
        <div
          onClick={() => toggleSection(type)}
          style={{
            cursor: 'pointer',
            padding: '8px',
            fontWeight: 'bold',
            background: '#f0f0f0',
            borderRadius: '4px',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <span>
            {icon} {type.charAt(0).toUpperCase() + type.slice(1)} ({items.length})
          </span>
          <span>{isExpanded ? '▼' : '▶'}</span>
        </div>
        {isExpanded && (
          <div style={{ marginLeft: '16px', marginTop: '4px' }}>
            {items.length === 0 ? (
              <div style={{ padding: '8px', color: '#999' }}>No items</div>
            ) : (
              items.map((item) => (
                <div
                  key={item.id}
                  onClick={() => onSelectResource(type, item.id)}
                  style={{
                    padding: '6px 8px',
                    cursor: 'pointer',
                    background:
                      selectedResource?.type === type && selectedResource?.id === item.id
                        ? '#e3f2fd'
                        : 'transparent',
                    borderRadius: '4px',
                    marginBottom: '2px',
                  }}
                  onMouseEnter={(e) => {
                    if (!(selectedResource?.type === type && selectedResource?.id === item.id)) {
                      e.currentTarget.style.background = '#f5f5f5';
                    }
                  }}
                  onMouseLeave={(e) => {
                    if (!(selectedResource?.type === type && selectedResource?.id === item.id)) {
                      e.currentTarget.style.background = 'transparent';
                    }
                  }}
                >
                  <div style={{ fontSize: '14px' }}>{item.name || `${type}_${item.id}`}</div>
                  <div style={{ fontSize: '11px', color: '#666' }}>
                    ID: {item.id} | Size: {(item.size / 1024).toFixed(2)} KB
                  </div>
                </div>
              ))
            )}
          </div>
        )}
      </div>
    );
  };

  return (
    <div
      style={{
        padding: '16px',
        height: '100%',
        overflow: 'auto',
        background: '#fafafa',
        borderRight: '1px solid #ddd',
      }}
    >
      <h3 style={{ marginTop: 0 }}>Resources</h3>
      {renderResourceList('images', resources.images, '🖼️')}
      {renderResourceList('sounds', resources.sounds, '🔊')}
      {renderResourceList('sprites', resources.sprites, '🎬')}
      {renderResourceList('scripts', resources.scripts, '⚡')}
    </div>
  );
};
