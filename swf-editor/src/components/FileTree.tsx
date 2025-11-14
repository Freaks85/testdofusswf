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

  const renderResourceList = (type: ResourceType, items: ResourceInfo[], icon: string, badgeClass: string) => {
    const isExpanded = expandedSections.has(type);

    return (
      <div key={type} style={{ marginBottom: '8px' }}>
        <div
          onClick={() => toggleSection(type)}
          className="cursor-pointer rounded flex justify-between items-center"
          style={{
            padding: '10px 12px',
            background: 'var(--bg-tertiary)',
            fontWeight: '600',
          }}
        >
          <span className="flex items-center gap-2">
            <span>{icon}</span>
            <span>{type.charAt(0).toUpperCase() + type.slice(1)}</span>
            <span className={`badge ${badgeClass}`}>{items.length}</span>
          </span>
          <span style={{ fontSize: '12px' }}>{isExpanded ? '▼' : '▶'}</span>
        </div>
        {isExpanded && (
          <div style={{ marginLeft: '12px', marginTop: '4px' }}>
            {items.length === 0 ? (
              <div className="p-2 text-muted">No items</div>
            ) : (
              items.map((item) => {
                const isSelected = selectedResource?.type === type && selectedResource?.id === item.id;
                return (
                  <div
                    key={item.id}
                    onClick={() => onSelectResource(type, item.id)}
                    className="cursor-pointer rounded"
                    style={{
                      padding: '8px 10px',
                      marginBottom: '2px',
                      background: isSelected ? 'var(--selection-bg)' : 'transparent',
                      borderLeft: isSelected ? '3px solid var(--border-focus)' : '3px solid transparent',
                      transition: 'all 0.2s ease',
                    }}
                    onMouseEnter={(e) => {
                      if (!isSelected) {
                        e.currentTarget.style.background = 'var(--bg-tertiary)';
                      }
                    }}
                    onMouseLeave={(e) => {
                      if (!isSelected) {
                        e.currentTarget.style.background = 'transparent';
                      }
                    }}
                  >
                    <div style={{ fontSize: '14px' }} className="text-primary">
                      {item.name || `${type}_${item.id}`}
                    </div>
                    <div style={{ fontSize: '11px' }} className="text-muted">
                      ID: {item.id} | Size: {(item.size / 1024).toFixed(2)} KB
                    </div>
                  </div>
                );
              })
            )}
          </div>
        )}
      </div>
    );
  };

  return (
    <div
      className="p-4 h-full overflow-auto"
      style={{
        background: 'var(--bg-secondary)',
      }}
    >
      <h3 style={{ marginTop: 0, marginBottom: '16px' }}>Resources</h3>
      {renderResourceList('images', resources.images, '🖼️', 'badge-image')}
      {renderResourceList('sounds', resources.sounds, '🔊', 'badge-sound')}
      {renderResourceList('sprites', resources.sprites, '🎬', 'badge-sprite')}
      {renderResourceList('scripts', resources.scripts, '⚡', 'badge-script')}
    </div>
  );
};
